use super::Scene;
use crate::geom::{Sphere, Volume};
use crate::material::{Background, DiffuseLight};
use crate::math::{Num, V3};
use crate::world::{Camera, World};
use crate::InputCollection;

pub struct Eve {
    aspect_ratio: f32,
}

impl Eve {
    pub fn new(aspect_ratio: f32) -> Self {
        Self { aspect_ratio }
    }
}

impl Scene for Eve {
    type Background = Box<dyn Background>;

    fn generate(
        &mut self,
        _animation_t: f32,
        _frame: u32,
        _input: &InputCollection,
    ) -> (World<Self::Background>, Camera) {
        use crate::eve;
        let cube_map = eve::environment("wormhole_class_05", V3::zero());
        let mut world = World::new(Box::new(cube_map) as Self::Background);

        let venture = eve::load_ship(eve::Hull::Stratios);
        let orca = eve::load_ship(eve::Hull::Nestor);

        let orca_pos = V3::new(-1250.0, 5.0, 0.0);
        world.add(orca.instance(
            orca_pos,
            V3::zero() + ((V3::rand() - 0.5) / 60.0),
            V3::one(),
        ));

        world.add(Volume::new(
            Sphere::new((), orca_pos, 1700.0),
            0.0006,
            V3::fill(0.4),
        ));

        let sun = Sphere::new(
            DiffuseLight::new(V3::new(4.0, 4.0, 5.0) * 10.0),
            V3::new(10000.0, -4000.0, 4800.0),
            1500.0,
        );
        world.add(sun);

        let look_from = V3::new(0.0, -20.0, 500.0);
        let rotation = V3::new(-0.03, 0.0, 0.0);

        for x in 0..6 {
            for z in 0..6 {
                let x = (x as f32 - 3.0) * 190.0;
                let z = (z as f32 - 3.0) * 190.0;
                let y = (f32::rand() * 2.0 - 1.0) * 150.0;
                let pos = V3::new(x, y, z);
                if pos.distance(look_from) > 50.0 {
                    let instance = venture.instance(
                        pos,
                        rotation + ((V3::rand() - 0.5) / 30.5),
                        V3::fill(0.2),
                    );
                    world.add(instance);
                }
            }
        }

        /*
        world.add(Sphere::new(
            Metal::new(0.0, V3::fill(1.0)),
            V3::zero(),
            100.0,
        ));
        */

        let look_at = orca_pos;

        let focus_distance = (look_from - look_at).length();
        let aperture = 0.2;

        let camera = Camera::new(
            50.0,
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
