#![allow(dead_code)]

use super::geom::{Model, Sphere, Triangle};
use super::material::{
    Background, CubeMap, Dielectric, DiffuseLight, Lambertian, Metal, Mix, SkyBackground,
    SolidBackground, Specular,
};
use super::obj_loader::ObjLoader;
use super::ply_loader::PlyLoader;
use super::world::{Camera, World};

use crate::math::{Num, V2, V3};

pub fn cornell_box(_animation_t: f32, aspect_ratio: f32) -> (World<impl Background>, Camera) {
    let mut world = World::new(SolidBackground::new(V3::fill(0.0)));

    let red = Lambertian::new(V3::new(1.0, 0.0, 0.0));
    let green = Lambertian::new(V3::new(0.0, 1.0, 0.0));
    let white = Lambertian::new(V3::new(1.0, 1.0, 1.0));
    let light = DiffuseLight::new(V3::fill(8.0));
    let sphere_material = Dielectric::new(1.3);

    // Once instance scaling is functional these models and be condensed into one
    let big_cube = PlyLoader::load(
        "cube.ply",
        |x, y, z| V3::new(x, y, z) * V3::fill(5.0),
        |a, b, c| Triangle::new((), a, b, c),
    )
    .unwrap();

    let light_cube = PlyLoader::load(
        "cube.ply",
        |x, y, z| V3::new(x, y, z) * V3::new(1.0, 0.0001, 1.0),
        |a, b, c| Triangle::new((), a, b, c),
    )
    .unwrap();

    let scene_cube = PlyLoader::load(
        "cube.ply",
        |x, y, z| V3::new(x, y, z) * V3::new(1.5, 3.0, 1.5),
        |a, b, c| Triangle::new((), a, b, c),
    )
    .unwrap();

    let big_cube = Model::new((), big_cube);
    let light_cube = Model::new((), light_cube);
    let scene_cube = Model::new((), scene_cube);

    world.add(big_cube.instance(red, V3::new(-10.0, 5.0, 0.0), V3::fill(0.0), V3::fill(1.0)));
    world.add(big_cube.instance(green, V3::new(10.0, 5.0, 0.0), V3::fill(0.0), V3::fill(1.0)));
    world.add(big_cube.instance(white, V3::new(0.0, 15.0, 0.0), V3::fill(0.0), V3::fill(1.0)));
    world.add(big_cube.instance(
        white,
        V3::new(0.0, 5.0, -10.0),
        V3::fill(0.0),
        V3::fill(1.0),
    ));
    world.add(big_cube.instance(
        white,
        V3::new(0.0, -5.0, -0.0),
        V3::fill(0.0),
        V3::fill(1.0),
    ));

    world.add(Sphere::new(sphere_material, V3::new(2.0, 2.0, 2.25), 2.0));

    world.add(light_cube.instance(
        light,
        V3::new(0.0, 10.0 - 0.00011, 0.0),
        V3::fill(0.0),
        V3::fill(1.0),
    ));

    world.add(scene_cube.instance(
        white,
        V3::new(-2.0, 3.0, -1.0),
        V3::new(0.0, -0.05, 0.0),
        V3::fill(1.0),
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

pub fn scratchpad(animation_t: f32, aspect_ratio: f32) -> (World<impl Background>, Camera) {
    let cube_map = crate::eve::environment("j01", V3::new(0.0, 0.0, 0.0));
    let mut world = World::new(cube_map);

    let ship = crate::eve::Ship::new("venture");

    let sun = Sphere::new(
        DiffuseLight::new(V3::new(4.0, 3.0, 1.0) * 10.0),
        V3::new(-10000.0, 0.0, -10000.0),
        1500.0,
    );
    //world.add(sun);

    let material = ship.material.clone();
    let list = ObjLoader::load(
        ship.model,
        |x, y, z| V3::new(x, y, z) * ship.scale,
        |x, y, z| V3::new(x, y, z),
        |u, v| V2::new(u, v),
        |a, b, c| Triangle::with_norms_and_uvs(material.clone(), a, b, c),
    )
    .unwrap();
    let model = Model::new(material.clone(), list);

    let look_from = V3::new(-230.0, 25.0, -230.0);

    let rotation = V3::new(-0.03, animation_t, 0.0);
    for x in 0..10 {
        for z in 0..10 {
            let x = (x as f32 - 5.0) * 55.0;
            let z = (z as f32 - 5.0) * 55.0;
            let y = (f32::rand() * 2.0 - 1.0) * 100.0;
            let pos = V3::new(x, y, z);
            if pos.distance(look_from) > 50.0 {
                let instance = model.instance(
                    material.clone(),
                    pos,
                    rotation + (V3::rand() / 35.0),
                    V3::fill(1.0),
                );
                world.add(instance);
            }
        }
    }

    /*
    world.add(Sphere::new(
        Metal::new(0.0, V3::fill(1.0)),
        V3::zero(),
        25.0,
    ));
    */

    let look_at = V3::new(0.0, 0.0, 0.0);
    let focus_distance = (look_from - look_at).length();
    let aperture = 0.2;

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
