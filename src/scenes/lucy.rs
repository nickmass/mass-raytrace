use super::Scene;
use crate::geom::{Model, Sphere, Triangle};
use crate::material::{DiffuseLight, Lambertian, SolidBackground};
use crate::math::{Num, V3, V4};
use crate::ply_loader::PlyLoader;
use crate::texture::SolidColor;
use crate::world::{Camera, World};
use crate::InputCollection;

pub struct Lucy {
    aspect_ratio: f32,
}

impl Lucy {
    pub fn new(aspect_ratio: f32) -> Self {
        Self { aspect_ratio }
    }
}

impl Scene for Lucy {
    type Background = SolidBackground;

    fn generate(
        &mut self,
        _animation_t: f32,
        _frame: u32,
        _input: &InputCollection,
    ) -> (World<Self::Background>, Camera) {
        let mut world = World::new(SolidBackground::new(V3::zero()));

        let mut max_dim = 0.0;

        let lucy = PlyLoader::load(
            "models/lucy.ply",
            |x, y, z| {
                max_dim = max_dim.max(x.abs()).max(y.abs()).max(z.abs());
                V3::new(y, z, x)
            },
            |a, b, c| Triangle::new((), a, b, c),
        )
        .unwrap();
        let lucy = Model::new(lucy);

        let white = Lambertian::new(SolidColor(V4::one()));
        let cube =
            PlyLoader::load("cube.ply", V3::new, |a, b, c| Triangle::new((), a, b, c)).unwrap();
        let cube = Model::new(cube);
        let ground = cube
            .instance(V3::new(0.0, -1000.0, 0.0), V3::zero(), V3::fill(1000.0))
            .with_material(white);

        world.add(ground);
        for x in -5..6 {
            for z in -5..6 {
                let material = Lambertian::new(SolidColor(V4::new(
                    1.0 - (f32::rand() * 0.5),
                    1.0 - (f32::rand() * 0.5),
                    1.0 - (f32::rand() * 0.5),
                    1.0,
                )));
                world.add(
                    lucy.instance(
                        V3::new(x as f32 * 3.0, 1.0, z as f32 * 3.0),
                        V3::new(0.0, f32::rand(), 0.0),
                        V3::fill(1.0 / max_dim) * 2.0,
                    )
                    .with_material(material),
                );
            }
        }

        let sun = Sphere::new(
            DiffuseLight::new(V3::new(4.0, 4.0, 5.0) * 10.0),
            V3::new(10000.0, 4000.0, 4800.0),
            1500.0,
        );
        world.add(sun);

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
