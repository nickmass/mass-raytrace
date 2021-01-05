#![allow(dead_code)]

use super::geom::{Model, Sphere, Triangle};
use super::material::{
    Background, Dielectric, DiffuseLight, Lambertian, Metal, SkyBackground, SolidBackground,
    Specular,
};
use super::ply_loader::PlyLoader;
use super::world::{Camera, World};
use crate::{
    math::{Num, V3},
    stl_loader,
};

pub fn cornell_box<B: Background>(world: &mut World<B>, aspect_ratio: f32) -> Camera {
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

    Camera::new(
        37.0,
        look_from,
        look_at,
        V3::new(0.0, 1.0, 0.0),
        aspect_ratio,
        aperture,
        focus_distance,
    )
}

pub fn scratchpad<B: Background>(world: &mut World<B>, aspect_ratio: f32) -> Camera {
    let mut max = V3::fill(f32::MIN);
    let mut min = V3::fill(f32::MAX);
    let scale = 25.0;
    let list = PlyLoader::load(
        "models/bun_zipper.ply",
        |x, y, z| {
            let v = V3::new(x, y, z) * scale;
            max = max.max(v);
            min = min.min(v);
            v
        },
        |a, b, c| Triangle::new((), a, b, c),
    )
    .unwrap();

    let model = Model::new((), list);

    let mut z = -50.0;

    for _ in 0..200 {
        let mut x = -50.0;
        for _ in 0..200 {
            let color = f32::rand();
            let color = V3::new(color, 1.0, 0.7).hsl_to_rgb();
            match fastrand::u8(0..4) {
                0 => {
                    let m = Specular::new(1.3, color);
                    let instance = model.instance(
                        m,
                        V3::new(x, min.y() * -1.0, z),
                        V3::new(0.0, f32::rand(), 0.0),
                        V3::fill(1.0),
                    );
                    world.add(instance);
                }
                1 => {
                    let m = Lambertian::new(color);
                    let instance = model.instance(
                        m,
                        V3::new(x, min.y() * -1.0, z),
                        V3::new(0.0, f32::rand(), 0.0),
                        V3::fill(1.0),
                    );
                    world.add(instance);
                }
                2 => {
                    let m = Metal::new(f32::rand(), color);
                    let instance = model.instance(
                        m,
                        V3::new(x, min.y() * -1.0, z),
                        V3::new(0.0, f32::rand(), 0.0),
                        V3::fill(1.0),
                    );
                    world.add(instance);
                }
                3 => {
                    let m = Dielectric::new(1.3);
                    let instance = model.instance(
                        m,
                        V3::new(x, min.y() * -1.0, z),
                        V3::new(0.0, f32::rand(), 0.0),
                        V3::fill(1.0),
                    );
                    world.add(instance);
                }
                _ => unreachable!(),
            };

            x += 5.0;
        }
        z += 5.0;
    }

    let look_from = V3::new(-55.0, 15.0, -55.0);
    let look_at = V3::new(0.0, 1.0, 0.0);
    let focus_distance = (look_from - look_at).length();
    let aperture = 0.2;

    Camera::new(
        37.0,
        look_from,
        look_at,
        V3::new(0.0, 1.0, 0.0),
        aspect_ratio,
        aperture,
        focus_distance,
    )
}

pub fn empty_world() -> World<SolidBackground> {
    World::new(SolidBackground::new(V3::fill(0.0)))
}

pub fn dark_world() -> World<SolidBackground> {
    let mut world = World::new(SolidBackground::new(V3::fill(0.0)));

    let material_ground = Lambertian::new(V3::fill(0.1));

    let world_sphere = Sphere::new(material_ground, V3::new(0.0, -10000.0001, 0.0), 10000.0);
    world.add(world_sphere);

    let sun_sphere = Sphere::new(
        DiffuseLight::new(V3::new(2.0, 2.0, 2.0)),
        V3::new(1100.0, 0.0, 1100.0),
        1000.0,
    );
    world.add(sun_sphere);

    world
}

pub fn bright_world() -> World<SkyBackground> {
    let mut world = World::new(SkyBackground);

    let material_ground = Lambertian::new(V3::fill(0.1));
    let material_ground = Metal::new(0.01, V3::fill(0.3));

    let world_sphere = Sphere::new(material_ground, V3::new(0.0, -100000.000000, 0.0), 100000.0);
    world.add(world_sphere);

    world
}
