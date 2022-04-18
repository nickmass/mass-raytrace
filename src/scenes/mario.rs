use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use gilrs::{Axis, Button};
use libsm64::{DynamicSurface, LevelTriangle, MarioInput, Sm64};
use winit::event::VirtualKeyCode;

use crate::geom::{Model, Triangle};
use crate::material::{Dielectric, Lambertian, Material, SkySphere};
use crate::math::{Num, M4, V2, V3, V4};
use crate::obj_loader::{ObjLoader, SimpleTexturedBuilder};
use crate::ply_loader::PlyLoader;
use crate::texture::{SharedTexture, SolidColorFallback, Surface, Texture, WrapMode};
use crate::world::{Camera, World};
use crate::{Input, InputCollection};

use std::io::Cursor;
use std::sync::Arc;

const COLLISION_LEVEL_SCALE: f32 = 1000.0;

pub struct Mario {
    aspect_ratio: f32,
    read_input: bool,
    write_input: bool,
    input_buf: Cursor<Vec<u8>>,
    output_buf: Vec<u8>,
    sm64: Sm64,
    platform: DynamicSurface,
    handle: libsm64::Mario,
    last_pos: V3,
    texture: SharedTexture,
    castle_triangles: Vec<Triangle<Lambertian<Arc<dyn Surface>>>>,
    platform_triangles: Vec<Triangle<()>>,
    sky_texture: SharedTexture,
}

impl Mario {
    pub fn new(aspect_ratio: f32, read_input: bool, write_input: bool) -> Self {
        let input_buf = if read_input {
            std::fs::read("models/mario/record_input.bin").unwrap()
        } else {
            Vec::new()
        };

        let input_buf = Cursor::new(input_buf);
        let output_buf = Vec::new();

        let rom = std::fs::File::open(std::env::var("SM64_ROM_PATH").unwrap()).unwrap();
        let mut sm64 = Sm64::new(rom).unwrap();
        let texture = sm64.texture();
        let texture =
            Texture::load_bytes(texture.data, texture.width, texture.height, WrapMode::Clamp)
                .shared();

        let builder = SimpleTexturedBuilder::new(WrapMode::Repeat);
        let castle_triangles =
            ObjLoader::load("models/mario/castle/Peaches Castle.obj", builder).unwrap();
        let castle_scale = M4::scale(V3::fill(COLLISION_LEVEL_SCALE));
        let castle_geo = castle_triangles
            .iter()
            .map(|triangle| create_level_triangle(triangle, castle_scale, false))
            .collect::<Vec<_>>();

        sm64.load_level_geometry(castle_geo.as_slice());

        let platform_triangles =
            PlyLoader::load("cube.ply", V3::new, |a, b, c| Triangle::new((), a, b, c)).unwrap();
        let platform_scale = V3::new(1.0, 0.1, 0.3);
        let platform_transform = M4::scale(platform_scale * COLLISION_LEVEL_SCALE);
        let platform_geo = platform_triangles
            .iter()
            .map(|triangle| create_level_triangle(triangle, platform_transform, true))
            .collect::<Vec<_>>();
        let platform_position = V3::new(1.4, 1.0, -1.0) * COLLISION_LEVEL_SCALE;
        let platform_transform = libsm64::SurfaceTransform {
            position: libsm64::Point3 {
                x: platform_position.x(),
                y: platform_position.y(),
                z: platform_position.z(),
            },
            euler_rotation: libsm64::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };
        let platform = sm64.create_dynamic_surface(&*platform_geo, platform_transform);

        let handle = sm64.create_mario(1100, 100, -4310).unwrap();

        let sky_texture = Texture::load_png("models/mario/mario_sky.png", WrapMode::Clamp)
            .unwrap()
            .shared();

        Self {
            aspect_ratio,
            read_input,
            write_input,
            input_buf,
            output_buf,
            handle,
            sm64,
            last_pos: V3::zero(),
            texture,
            platform,
            castle_triangles,
            platform_triangles,
            sky_texture,
        }
    }
}

impl Drop for Mario {
    fn drop(&mut self) {
        if self.write_input {
            println!("Writing output buf");
            std::fs::write("models/mario/record_input.bin", &self.output_buf).unwrap();
        }
    }
}

impl super::Scene for Mario {
    type Background = SkySphere<SharedTexture>;

    fn generate(
        &mut self,
        _animation_t: f32,
        frame: u32,
        input: &InputCollection,
    ) -> (World<Self::Background>, Camera) {
        let sky = SkySphere::new(self.sky_texture.clone());
        let mut world = World::new(sky);

        let castle = Model::new(self.castle_triangles.clone());
        world.add(castle);

        let look_from = V3::new(0.4, 1.4455.max(self.last_pos.y() + 0.3), -1.0005);

        let platform_scale = V3::new(1.0, 0.1, 0.3);
        let platform_position = V3::new(3.4, 1.3 + ((frame as f32 / 30.0).sin() / 0.8), -1.0);
        let platform_position_scaled = platform_position * COLLISION_LEVEL_SCALE;
        let platform_rotation = frame as f32 / 380.0;

        let platform_transform = libsm64::SurfaceTransform {
            position: libsm64::Point3 {
                x: platform_position_scaled.x(),
                y: platform_position_scaled.y(),
                z: platform_position_scaled.z(),
            },
            euler_rotation: libsm64::Point3 {
                x: 0.0,
                y: platform_rotation * 360.0,
                z: 0.0,
            },
        };
        self.platform.transform(platform_transform);

        let cube = Model::new(self.platform_triangles.clone());
        world.add(
            cube.instance(
                platform_position,
                V3::new(0.0, platform_rotation, 0.0),
                platform_scale,
            )
            .with_material(Dielectric::new(1.7)),
        );

        let mut mario_input = MarioInput::default();
        if self.read_input {
            mario_input.from_bytes(&mut self.input_buf).unwrap();
        } else {
            mario_input.button_a = input.is_pressed(Input::Key(VirtualKeyCode::J))
                || input.is_pressed(Input::Button(Button::South));
            mario_input.button_b = input.is_pressed(Input::Key(VirtualKeyCode::K))
                || input.is_pressed(Input::Button(Button::East));
            mario_input.button_z = input.is_pressed(Input::Key(VirtualKeyCode::L))
                || input.is_pressed(Input::Button(Button::West));

            if input.is_pressed(Input::Key(VirtualKeyCode::W)) {
                mario_input.stick_y = -1.0;
            } else if input.is_pressed(Input::Key(VirtualKeyCode::S)) {
                mario_input.stick_y = 1.0;
            } else {
                mario_input.stick_y = input.axis(Axis::LeftStickY) * -1.0;
            }

            if input.is_pressed(Input::Key(VirtualKeyCode::A)) {
                mario_input.stick_x = -1.0;
            } else if input.is_pressed(Input::Key(VirtualKeyCode::D)) {
                mario_input.stick_x = 1.0;
            } else {
                mario_input.stick_x = input.axis(Axis::LeftStickX);
            }
        }
        if self.write_input {
            mario_input.to_bytes(&mut self.output_buf).unwrap();
        }

        mario_input.cam_look_x = self.last_pos.x() - look_from.x();
        mario_input.cam_look_z = self.last_pos.z() - look_from.z();

        let tex = self.texture.clone();

        let mario_state = self.handle.tick(mario_input);
        let scale = M4::scale(V3::fill(1.0 / COLLISION_LEVEL_SCALE));

        let mario_tris = self
            .handle
            .geometry()
            .triangles()
            .map(|mario_tri| {
                let color = V4::new(
                    mario_tri.0.color.r,
                    mario_tri.0.color.g,
                    mario_tri.0.color.b,
                    1.0,
                );

                let m_color = SolidColorFallback::new(color, tex.clone());
                let material = Lambertian::new(m_color);

                let to_v3 = |point: libsm64::Point3<f32>| V3::new(point.x, point.y, point.z);
                let to_v2 = |point: libsm64::Point2<f32>| V2::new(point.x, point.y);

                let v_a = to_v3(mario_tri.0.position);
                let v_b = to_v3(mario_tri.1.position);
                let v_c = to_v3(mario_tri.2.position);

                let v_a = scale.transform_point(v_a);
                let v_b = scale.transform_point(v_b);
                let v_c = scale.transform_point(v_c);

                let n_a = to_v3(mario_tri.0.normal);
                let n_b = to_v3(mario_tri.1.normal);
                let n_c = to_v3(mario_tri.2.normal);

                let uv_a = to_v2(mario_tri.0.uv);
                let uv_b = to_v2(mario_tri.1.uv);
                let uv_c = to_v2(mario_tri.2.uv);

                let a = (v_a, n_a, uv_a);
                let b = (v_b, n_b, uv_b);
                let c = (v_c, n_c, uv_c);

                Triangle::with_norms_and_uvs(material, a, b, c)
            })
            .collect::<Vec<_>>();

        let mario_pos = V3::new(
            mario_state.position.x,
            mario_state.position.y,
            mario_state.position.z,
        );
        let mario_pos = scale.transform_point(mario_pos);

        self.last_pos = mario_pos;

        let mario = Model::new(mario_tris);

        world.add(mario);

        let look_at = mario_pos;
        let focus_distance = (look_from - look_at).length();
        let aperture = 0.00;

        let camera = Camera::new(
            80.0,
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

fn create_level_triangle<M: Material>(
    triangle: &Triangle<M>,
    transform: M4,
    winding: bool,
) -> LevelTriangle {
    let mut verts = triangle.vertices();

    verts.0 = transform.transform_point(verts.0);
    verts.1 = transform.transform_point(verts.1);
    verts.2 = transform.transform_point(verts.2);

    let a = libsm64::Point3 {
        x: verts.0.x() as i16,
        y: verts.0.y() as i16,
        z: verts.0.z() as i16,
    };

    let b = libsm64::Point3 {
        x: verts.1.x() as i16,
        y: verts.1.y() as i16,
        z: verts.1.z() as i16,
    };

    let c = libsm64::Point3 {
        x: verts.2.x() as i16,
        y: verts.2.y() as i16,
        z: verts.2.z() as i16,
    };

    let vertices = if winding { (a, b, c) } else { (c, b, a) };

    LevelTriangle {
        kind: libsm64::Surface::Default,
        force: 0,
        terrain: libsm64::Terrain::Grass,
        vertices,
    }
}

pub trait MarioInputExt {
    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error>;
    fn from_bytes<R: std::io::Read>(&mut self, reader: &mut R) -> Result<(), std::io::Error>;
}

impl MarioInputExt for libsm64::MarioInput {
    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        writer.write_u8(self.button_a as u8)?;
        writer.write_u8(self.button_b as u8)?;
        writer.write_u8(self.button_z as u8)?;
        writer.write_f32::<LittleEndian>(self.stick_x)?;
        writer.write_f32::<LittleEndian>(self.stick_y)?;

        Ok(())
    }

    fn from_bytes<R: std::io::Read>(&mut self, reader: &mut R) -> Result<(), std::io::Error> {
        self.button_a = reader.read_u8()? != 0;
        self.button_b = reader.read_u8()? != 0;
        self.button_z = reader.read_u8()? != 0;
        self.stick_x = reader.read_f32::<LittleEndian>()?;
        self.stick_y = reader.read_f32::<LittleEndian>()?;

        Ok(())
    }
}
