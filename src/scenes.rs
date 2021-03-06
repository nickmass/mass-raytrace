#![allow(dead_code)]

use super::geom::{Model, Sphere, Triangle};
use super::material::{
    Background, CubeMap, Dielectric, DiffuseLight, Lambertian, Metal, Mix, SkyBackground,
    SolidBackground, Specular,
};
use super::ply_loader::PlyLoader;
use super::world::{Camera, World};

use crate::math::{Num, V2, V3};

pub fn cornell_box(_animation_t: f32, aspect_ratio: f32) -> (World<impl Background>, Camera) {
    let mut world = World::new(SolidBackground::new(V3::zero()));

    let red = Lambertian::new(V3::new(1.0, 0.0, 0.0));
    let green = Lambertian::new(V3::new(0.0, 1.0, 0.0));
    let white = Lambertian::new(V3::one());
    let light = DiffuseLight::new(V3::fill(8.0));
    let sphere_material = Dielectric::new(1.3);

    let cube = PlyLoader::load("cube.ply", V3::new, |a, b, c| Triangle::new((), a, b, c)).unwrap();

    let cube = Model::new((), cube);

    world.add(cube.instance(red, V3::new(-10.0, 5.0, 0.0), V3::zero(), V3::fill(5.0)));
    world.add(cube.instance(green, V3::new(10.0, 5.0, 0.0), V3::zero(), V3::fill(5.0)));
    world.add(cube.instance(white, V3::new(0.0, 15.0, 0.0), V3::zero(), V3::fill(5.0)));
    world.add(cube.instance(white, V3::new(0.0, 5.0, -10.0), V3::zero(), V3::fill(5.0)));
    world.add(cube.instance(white, V3::new(0.0, -5.0, -0.0), V3::zero(), V3::fill(5.0)));

    world.add(Sphere::new(sphere_material, V3::new(1.75, 2.0, 2.25), 2.0));

    world.add(cube.instance(
        light,
        V3::new(0.0, 10.0 - 0.00011, 0.0),
        V3::zero(),
        V3::new(1.0, 0.0001, 1.0),
    ));

    world.add(cube.instance(
        white,
        V3::new(-2.0, 3.0, -1.0),
        V3::new(0.0, -0.05, 0.0),
        V3::new(1.75, 3.1, 1.75),
    ));

    let look_from = V3::new(0.0, 5.0, 20.0);
    let look_at = V3::new(0.0, 5.0, 0.0);
    let focus_distance = (look_from - look_at).length();
    let aperture = 0.00;

    let camera = Camera::new(
        37.0,
        look_from,
        look_at,
        V3::new(0.0, 1.0, 0.0),
        aspect_ratio,
        aperture,
        focus_distance,
    );

    (world, camera)
}

pub fn scratchpad(_animation_t: f32, aspect_ratio: f32) -> (World<impl Background>, Camera) {
    use crate::eve;
    let cube_map = eve::environment("c01", V3::zero());
    let mut world = World::new(cube_map);

    let venture = eve::load_ship(eve::Hull::Venture);
    let orca = eve::load_ship(eve::Hull::Orca);

    let orca_pos = V3::new(-1250.0, 5.0, 0.0);
    world.add(orca.instance(
        orca.material.clone(),
        orca_pos,
        V3::zero() + ((V3::rand() - 0.5) / 60.0),
        V3::one(),
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
                    venture.material.clone(),
                    pos,
                    rotation + ((V3::rand() - 0.5) / 30.5),
                    V3::one(),
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
    let aperture = 6.0;

    let camera = Camera::new(
        50.0,
        look_from,
        look_at,
        V3::new(0.0, 1.0, 0.0),
        aspect_ratio,
        aperture,
        focus_distance,
    );

    (world, camera)
}
