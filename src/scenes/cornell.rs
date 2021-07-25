use super::Scene;
use crate::geom::{Model, Sphere, Triangle};
use crate::material::{Dielectric, DiffuseLight, Lambertian, SolidBackground};
use crate::math::{V3, V4};
use crate::ply_loader::PlyLoader;
use crate::texture::SolidColor;
use crate::world::{Camera, World};
use crate::InputCollection;

pub struct CornellBox {
    aspect_ratio: f32,
}

impl CornellBox {
    pub fn new(aspect_ratio: f32) -> Self {
        Self { aspect_ratio }
    }
}

impl Scene for CornellBox {
    type Background = SolidBackground;

    fn generate(
        &mut self,
        _animation_t: f32,
        _frame: u32,
        _input: &InputCollection,
    ) -> (World<Self::Background>, Camera) {
        let mut world = World::new(SolidBackground::new(V3::zero()));

        let red = Lambertian::new(SolidColor(V4::new(1.0, 0.0, 0.0, 1.0)));
        let green = Lambertian::new(SolidColor(V4::new(0.0, 1.0, 0.0, 1.0)));
        let white = Lambertian::new(SolidColor(V4::one()));
        let light = DiffuseLight::new(V3::fill(8.0));
        let sphere_material = Dielectric::new(1.3);

        let cube =
            PlyLoader::load("cube.ply", V3::new, |a, b, c| Triangle::new((), a, b, c)).unwrap();

        let cube = Model::new(cube);

        world.add(
            cube.instance(V3::new(-10.0, 5.0, 0.0), V3::zero(), V3::fill(5.0))
                .with_material(red),
        );
        world.add(
            cube.instance(V3::new(10.0, 5.0, 0.0), V3::zero(), V3::fill(5.0))
                .with_material(green),
        );
        world.add(
            cube.instance(V3::new(0.0, 15.0, 0.0), V3::zero(), V3::fill(5.0))
                .with_material(white),
        );
        world.add(
            cube.instance(V3::new(0.0, 5.0, -10.0), V3::zero(), V3::fill(5.0))
                .with_material(white),
        );
        world.add(
            cube.instance(V3::new(0.0, -5.0, -0.0), V3::zero(), V3::fill(5.0))
                .with_material(white),
        );

        world.add(Sphere::new(sphere_material, V3::new(1.75, 2.0, 2.25), 2.0));

        world.add(
            cube.instance(
                V3::new(0.0, 10.0 - 0.00011, 0.0),
                V3::zero(),
                V3::new(1.0, 0.0001, 1.0),
            )
            .with_material(light),
        );

        world.add(
            cube.instance(
                V3::new(-2.0, 3.0, -1.0),
                V3::new(0.0, -0.05, 0.0),
                V3::new(1.75, 3.1, 1.75),
            )
            .with_material(white),
        );

        let look_from = V3::new(0.0, 5.0, 20.0);
        let look_at = V3::new(0.0, 5.0, 0.0);
        let focus_distance = (look_from - look_at).length();
        let aperture = 0.00;

        let camera = Camera::new(
            37.0,
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
