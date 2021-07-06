use std::path::Path;
use std::sync::Arc;

use super::material::{
    Background, CubeMap, Dielectric, Lambertian, Material, Metal, Mix, Specular,
};

use crate::obj_loader::ObjGroupFilter;
use crate::texture::{Surface, Texture, YCbCrTexture};
use crate::{
    math::{V2, V3},
    texture::{BlendMode, TextureBlend},
};

pub struct EveFilter;

impl ObjGroupFilter for EveFilter {
    fn include_group(&self, group_name: Option<&str>) -> bool {
        if let Some(group_name) = group_name {
            let include = [
                "Hull", "hull", "Glass", "glass", "DarkHull", "exhaust", "Exhaust",
            ]
            .contains(&group_name);

            if !include {
                println!("obj discarding: {}", group_name);
            }

            include
        } else {
            false
        }
    }
}

struct InnerEveMaterial {
    normal_occlusion: Texture,
    albedo_roughness: Texture,
    pmdg: Texture,
    colors: EveMaterialColor,
}

#[derive(Clone)]
pub struct EveMaterial {
    inner: Arc<InnerEveMaterial>,
}

impl EveMaterial {
    pub fn new<P: AsRef<Path>>(
        no: P,
        ar: P,
        pmdg: P,
        colors: EveMaterialColor,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let normal_occlusion = Texture::load_png(no)?;
        let albedo_roughness = Texture::load_png(ar)?;
        let pmdg = Texture::load_png(pmdg)?;

        let inner = InnerEveMaterial {
            normal_occlusion,
            albedo_roughness,
            pmdg,
            colors,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn normal_occlusion(&self, uv: V2) -> (V3, f32) {
        let pixel = self.inner.normal_occlusion.get_f(uv);
        let occ = pixel.z();
        let pixel = pixel * 2.0 - 1.0;
        let x = 1.0 - pixel.y().powi(2) - pixel.w().powi(2);
        let z = x.abs().sqrt();
        (V3::new(pixel.y(), pixel.w(), z).unit(), occ)
    }

    pub fn albedo_roughness(&self, uv: V2) -> (V3, f32) {
        let pixel = self.inner.albedo_roughness.get_f(uv);
        (pixel.contract(), pixel.w())
    }

    pub fn pmdg(&self, uv: V2) -> (f32, f32, f32, f32) {
        let pixel = self.inner.pmdg.get_f(uv);
        let paint = pixel.x();
        let material = pixel.y();
        let dirt = pixel.z();
        let glow = pixel.w();

        (paint, material, dirt, glow)
    }

    pub fn dump_normals<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();

        let mut pixels = Vec::new();
        for y in 0..self.inner.normal_occlusion.height() {
            for x in 0..self.inner.normal_occlusion.width() {
                let uv = V2::new(
                    x as f32 / (self.inner.normal_occlusion.width() - 1) as f32,
                    y as f32 / (self.inner.normal_occlusion.height() - 1) as f32,
                );
                let (norm, _occ) = self.normal_occlusion(uv);

                let norm = ((norm + 1.0) / 2.0) * 255.0;

                pixels.push(norm.x() as u8);
                pixels.push(norm.y() as u8);
                pixels.push(norm.z() as u8);
            }
        }

        image::save_buffer_with_format(
            path,
            &pixels,
            self.inner.normal_occlusion.width(),
            self.inner.normal_occlusion.height(),
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .unwrap();
    }
}

impl Material for EveMaterial {
    fn scatter(
        &self,
        ray: crate::world::Ray,
        hit: &crate::geom::Hit,
    ) -> Option<crate::material::Scatter> {
        if let Some(uv) = hit.uv {
            let (albedo, roughness) = self.albedo_roughness(uv);
            let (paint, material, dirt, _glow) = self.pmdg(uv);

            let dirt = dirt * 1.0;

            let material_color = self.inner.colors.get(material);
            let color = (((albedo * material_color * (1.0 - paint)) + (albedo * paint))
                * (1.0 - dirt.min(1.0)))
                + (V3::new(0.01, 0.005, 0.0) * dirt);

            Mix::new(
                (roughness + dirt).min(1.0),
                Lambertian::new(color),
                Specular::new(1.8, color),
            )
            .scatter(ray, hit)
        } else {
            None
        }
    }

    fn emit(&self, hit: &crate::geom::Hit) -> Option<V3> {
        if let Some(uv) = hit.uv {
            let (_paint, _material, _dirt, glow) = self.pmdg(uv);
            Some(self.inner.colors.glow * glow * 10.0)
        } else {
            None
        }
    }

    fn normal(&self, uv: V2) -> Option<V3> {
        let (norm, _occ) = self.normal_occlusion(uv);
        Some(norm)
    }
}

pub struct EveMaterialColor {
    colors: [V3; 4],
    glow: V3,
}

impl EveMaterialColor {
    pub fn test() -> Self {
        Self {
            colors: [
                V3::new(1.0, 0.0, 0.0),
                V3::new(0.0, 1.0, 0.0),
                V3::new(0.0, 0.0, 1.0),
                V3::new(1.0, 0.0, 1.0),
            ],
            glow: V3::new(0.5, 0.85, 2.0),
        }
    }

    pub fn caldari() -> Self {
        Self {
            colors: [
                V3::new(0.02, 0.02, 0.02),
                V3::new(0.1, 0.1, 0.1),
                V3::new(0.03, 0.05, 0.1),
                V3::new(0.08, 0.08, 0.08),
            ],
            glow: V3::new(0.5, 0.85, 2.0),
        }
    }

    pub fn ore() -> Self {
        Self {
            colors: [
                V3::new(0.02, 0.02, 0.02),
                V3::new(0.1, 0.1, 0.1),
                V3::new(0.15, 0.26, 0.39),
                V3::new(0.85, 0.62, 0.2),
            ],
            glow: V3::new(0.5, 0.85, 2.0),
        }
    }

    pub fn soe() -> Self {
        Self {
            colors: [
                V3::new(0.02, 0.02, 0.02),
                V3::new(0.2, 0.2, 0.2),
                V3::new(1.0, 1.0, 1.0),
                V3::new(0.5, 0.0, 0.0),
            ],
            glow: V3::new(0.5, 0.85, 2.0),
        }
    }

    fn get(&self, i: f32) -> V3 {
        let i = i * 3.0;
        let i0 = i.floor() as usize;
        let i1 = i.ceil() as usize;

        let t = i - i0 as f32;

        self.colors[i0] * (1.0 - t) + self.colors[i1] * t
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Hull {
    Astero,
    Avatar,
    Buzzard,
    Crow,
    Nestor,
    Orca,
    Raven,
    Rifter,
    Stratios,
    Venture,
}

pub fn load_ship(hull: Hull) -> crate::geom::Model<EveMaterial> {
    let (material, model) = match hull {
        Hull::Venture => {
            let material = EveMaterial::new(
                "models/oref1_t1/oref1_t1_no.png",
                "models/oref1_t1/oref1_t1_ar.png",
                "models/oref1_t1/oref1_t1_pmdg.png",
                EveMaterialColor::ore(),
            )
            .unwrap();
            let model = "models/oref1_t1/OreF1_TShape1.obj";
            (material, model)
        }
        Hull::Raven => {
            let material = EveMaterial::new(
                "models/cb1_t1/cb1_t1_no.png",
                "models/cb1_t1/cb1_t1_ar.png",
                "models/cb1_t1/cb1_t1_navy_pmdg.png",
                EveMaterialColor::caldari(),
            )
            .unwrap();
            let model = "models/cb1_t1/CB1_TShape1.obj";
            (material, model)
        }
        Hull::Avatar => {
            let material = EveMaterial::new(
                "models/at1_t1/at1_t1_no.png",
                "models/at1_t1/at1_t1_ar.png",
                "models/at1_t1/at1_t1_pmdg.png",
                EveMaterialColor::ore(),
            )
            .unwrap();
            let model = "models/at1_t1/AT1_TShape1.obj";
            (material, model)
        }
        Hull::Buzzard => {
            let material = EveMaterial::new(
                "models/cf3_t2/cf3_t2_no.png",
                "models/cf3_t2/cf3_t2_ar.png",
                "models/cf3_t2/cf3_t2_pmdg.png",
                EveMaterialColor::caldari(),
            )
            .unwrap();
            let model = "models/cf3_t2/CF3_TShape2.obj";
            (material, model)
        }
        Hull::Rifter => {
            let material = EveMaterial::new(
                "models/mf4_t1/mf4_t1_no.png",
                "models/mf4_t1/mf4_t1_ar.png",
                "models/mf4_t1/mf4_t1_pmdg.png",
                EveMaterialColor::ore(),
            )
            .unwrap();
            let model = "models/mf4_t1/MF4_TShape1.obj";
            (material, model)
        }
        Hull::Astero => {
            let material = EveMaterial::new(
                "models/soef1_t1/soef1_t1_no.png",
                "models/soef1_t1/soef1_t1_ar.png",
                "models/soef1_t1/soef1_t1_pmdg.png",
                EveMaterialColor::soe(),
            )
            .unwrap();
            let model = "models/soef1_t1/SoEF1_TShape1.obj";
            (material, model)
        }
        Hull::Stratios => {
            let material = EveMaterial::new(
                "models/soec1_t1/soec1_t1_no.png",
                "models/soec1_t1/soec1_t1_ar.png",
                "models/soec1_t1/soec1_t1_pmdg.png",
                EveMaterialColor::soe(),
            )
            .unwrap();
            let model = "models/soec1_t1/SoEC1_TShape1.obj";
            (material, model)
        }
        Hull::Nestor => {
            let material = EveMaterial::new(
                "models/soeb1_t1/soeb1_t1_no.png",
                "models/soeb1_t1/soeb1_t1_ar.png",
                "models/soeb1_t1/soeb1_t1_pmdg.png",
                EveMaterialColor::soe(),
            )
            .unwrap();
            let model = "models/soeb1_t1/SoEB1_TShape2.obj";
            (material, model)
        }
        Hull::Orca => {
            let material = EveMaterial::new(
                "models/orefr1_t1/orefr1_t1_no.png",
                "models/orefr1_t1/orefr1_t1_ar.png",
                "models/orefr1_t1/orefr1_t1_pmdg.png",
                EveMaterialColor::ore(),
            )
            .unwrap();
            let model = "models/orefr1_t1/OreFr1_TShape1.obj";
            (material, model)
        }
        Hull::Crow => {
            let material = EveMaterial::new(
                "models/cf2_t2a/cf2_t2a_no.png",
                "models/cf2_t2a/cf2_t2a_ar.png",
                "models/cf2_t2a/cf2_t2a_pmdg.png",
                EveMaterialColor::ore(),
            )
            .unwrap();
            let model = "models/cf2_t2a/CF2_T2aShape.obj";
            (material, model)
        }
    };

    let tris = crate::obj_loader::ObjLoader::load(
        model,
        &EveFilter,
        V3::new,
        V3::new,
        V2::new,
        |a, b, c| crate::geom::Triangle::with_norms_and_uvs(material.clone(), a, b, c),
    )
    .unwrap();

    crate::geom::Model::new(material, tris)
}

pub fn environment(name: &str, rotation: V3) -> impl Background {
    let stars = Texture::load_png("models/environments/stars01_tile2.png")
        .unwrap()
        .shared();
    let cube_dir = |index| {
        let luma = format!("models/environments/{}/{}.png", name, index);
        let chroma = format!("models/environments/{}/{}_chroma.png", name, index);

        let stars = stars.clone();
        let nebula = YCbCrTexture::load_png(luma, chroma).unwrap();

        TextureBlend::new(BlendMode::Addition, stars, nebula)
    };
    CubeMap::new(
        cube_dir(0),
        cube_dir(1),
        cube_dir(2),
        cube_dir(3),
        cube_dir(4),
        cube_dir(5),
        rotation,
    )
}
