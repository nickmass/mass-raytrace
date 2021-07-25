use super::Scene;
use crate::geom::{Model, Sphere, Triangle};
use crate::material::{Dielectric, DiffuseLight, Lambertian, Metal, SolidBackground};
use crate::math::{V3, V4};
use crate::ply_loader::PlyLoader;
use crate::texture::SolidColor;
use crate::world::{Camera, World};
use crate::InputCollection;

pub struct SphereGrid {
    aspect_ratio: f32,
}

impl SphereGrid {
    pub fn new(aspect_ratio: f32) -> Self {
        Self { aspect_ratio }
    }
}

impl Scene for SphereGrid {
    type Background = SolidBackground;

    fn generate(
        &mut self,
        _animation_t: f32,
        _frame: u32,
        _input: &InputCollection,
    ) -> (World<Self::Background>, Camera) {
        let mut world = World::new(SolidBackground::new(V3::zero()));

        let white = Lambertian::new(SolidColor(V4::one()));
        let cube =
            PlyLoader::load("cube.ply", V3::new, |a, b, c| Triangle::new((), a, b, c)).unwrap();
        let cube = Model::new(cube);
        let ground = cube
            .instance(V3::new(0.0, -1000.0, 0.0), V3::zero(), V3::fill(1000.0))
            .with_material(white);

        world.add(ground);

        let r: f32 = 1.0;
        let d = r * 2.0;
        let a = (d.powi(2) - r.powi(2)).sqrt();

        let dim = 50;
        for i in -dim..dim {
            for j in -dim..dim {
                let off = if j % 2 == 0 { r } else { 0.0 };
                let x = (i as f32 * d) + off;
                let z = j as f32 * a;
                let y = r;

                let r = r - 0.05;

                match (i, j) {
                    (0, 0) => {
                        let m = DiffuseLight::new(V3::fill(3.0));
                        let s = Sphere::new(m, V3::new(x, y, z), r);

                        world.add(s);
                    }
                    (-1, 0) | (1, 0) | (1, -1) | (0, -1) | (1, 1) | (0, 1) => {
                        let m = Dielectric::new(1.8);
                        let s = Sphere::new(m, V3::new(x, y, z), r);

                        world.add(s);
                    }
                    (_, _) => {
                        let m = Metal::new(0.0, SolidColor(V3::rand().expand(1.0)));
                        let s = Sphere::new(m, V3::new(x, y, z), r);

                        world.add(s);
                    }
                }
            }
        }

        let look_from = V3::new(6.0, 8.0, 5.0);
        let look_at = V3::new(0.0, 0.0, 0.0);
        let focus_distance = (look_from - look_at).length();
        let aperture = 0.00;

        let camera = Camera::new(
            40.0,
            look_from,
            look_at,
            V3::new(0.0, 1.0, 0.0),
            self.aspect_ratio,
            aperture,
            focus_distance,
        );

        (world, camera)
    }
}
