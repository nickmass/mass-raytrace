use super::Scene;
use crate::geom::{Model, Triangle};
use crate::material::{Background, Lambertian, Metal};
use crate::math::{Num, V3, V4};
use crate::ply_loader::PlyLoader;
use crate::texture::SolidColor;
use crate::world::{Camera, World};
use crate::InputCollection;

pub struct Menger {
    aspect_ratio: f32,
}

impl Menger {
    pub fn new(aspect_ratio: f32) -> Self {
        Self { aspect_ratio }
    }
}

impl Scene for Menger {
    type Background = Box<dyn Background>;

    fn generate(
        &mut self,
        _animation_t: f32,
        _frame: u32,
        _input: &InputCollection,
    ) -> (World<Self::Background>, Camera) {
        let cube_map = crate::eve::environment("j02", V3::new(0.4, 0.2, 0.1));
        let mut world = World::new(Box::new(cube_map) as Self::Background);

        let cube =
            PlyLoader::load("cube.ply", V3::new, |a, b, c| Triangle::new((), a, b, c)).unwrap();
        let cube = Model::new(cube);

        let foggy = Metal::new(0.7, SolidColor(V3::fill(0.5).expand(1.0)));

        menger_gen(&mut world);

        world.add(
            cube.instance(
                V3::new(0.0, -244.0, 0.0),
                V3::zero(),
                V3::new(500000.0, 1.0, 500000.0),
            )
            .with_material(foggy),
        );

        let look_from = V3::new(2680.0, 140.0, 2000.0);
        let look_at = V3::new(0.0, 0.0, 0.0);
        let focus_distance = (look_from - look_at).length();
        let aperture = 0.00;

        let camera = Camera::new(
            15.0,
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

fn menger_gen(world: &mut World<impl Background>) {
    let dims = 2.0;
    let material = Lambertian::new(SolidColor(V4::fill(1.0)));
    let cube = PlyLoader::load("cube.ply", V3::new, |a, b, c| Triangle::new((), a, b, c)).unwrap();
    let cube = Model::new(cube);
    let mut min = 0.0;
    let mut max = 0.0;
    let mut add_cube = |xyz| {
        let cube = cube
            .instance(xyz, V3::zero(), V3::one())
            .with_material(material);
        min = min.min(xyz.y());
        max = max.max(xyz.y());
        world.add(cube);
    };
    for (i, j, k) in MENGER_CUBE_SIDES.iter().copied() {
        let xyz = V3::new(i as f32, j as f32, k as f32) * dims * (3.0f32).powi(4);
        for (i, j, k) in MENGER_CUBE_SIDES.iter().copied() {
            let xyz = V3::new(i as f32, j as f32, k as f32) * dims * (3.0f32).powi(3) + xyz;
            for (i, j, k) in MENGER_CUBE_SIDES.iter().copied() {
                let xyz = (V3::new(i as f32, j as f32, k as f32) * dims * (3.0f32).powi(2)) + xyz;
                for (i, j, k) in MENGER_CUBE_SIDES.iter().copied() {
                    let xyz =
                        (V3::new(i as f32, j as f32, k as f32) * dims * (3.0f32).powi(1)) + xyz;
                    for (i, j, k) in MENGER_CUBE_SIDES.iter().copied() {
                        let xyz =
                            (V3::new(i as f32, j as f32, k as f32) * dims * (3.0f32).powi(0)) + xyz;
                        add_cube(xyz);
                    }
                }
            }
        }
    }
}

const MENGER_CUBE_SIDES: &[(i32, i32, i32)] = &[
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
    (0, -1, -1),
    (-1, 0, -1),
    (-1, -1, 0),
    (0, -1, 1),
    (-1, 0, 1),
    (-1, 1, 0),
    (0, 1, -1),
    (1, 0, -1),
    (1, -1, 0),
    (-1, -1, 1),
    (-1, 1, -1),
    (1, -1, -1),
    (-1, 1, 1),
    (1, -1, 1),
    (1, 1, -1),
    (1, 1, 1),
    (-1, -1, -1),
];
