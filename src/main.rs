use glium::backend::Facade;
use glium::glutin;
use glium::texture::SrgbTexture2d;
use glium::{implement_vertex, uniform, DrawParameters, Program, Surface};
use glutin::event_loop::EventLoopProxy;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::sync::{Arc, Mutex};

mod math;
use math::{Num, V3};

mod geom;
mod material;
mod ply_loader;
mod scenes;
mod world;

#[derive(Debug)]
enum UserEvent {
    Update,
}

const MAX_DEPTH: i32 = 250;

const ASPECT_RATIO: f64 = 16.0 / 9.0;

const IMAGE_WIDTH: u32 = 1920 * 2;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;

fn main() {
    fastrand::seed(1);

    let mut world = scenes::empty_world();
    let camera = scenes::cornell_box(&mut world, ASPECT_RATIO);

    world.build_bvh();

    let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event();
    let image = Arc::new(Image::new(IMAGE_WIDTH, IMAGE_HEIGHT));

    {
        let image = image.clone();
        let proxy = Arc::new(Mutex::new(event_loop.create_proxy()));
        render(image, proxy, world, camera);
    }

    run(event_loop, image)
}

fn render<B: 'static + material::Background>(
    image: Arc<Image>,
    event_proxy: Arc<Mutex<EventLoopProxy<UserEvent>>>,
    world: world::World<B>,
    camera: world::Camera,
) {
    let world = Arc::new(world);
    let camera = Arc::new(camera);
    let cpus = num_cpus::get() as i32;
    let cpus = (cpus - 2).max(1);
    for i in 0..cpus {
        let event_proxy = event_proxy.clone();
        let world = world.clone();
        let camera = camera.clone();
        let image = image.clone();
        let mut buffer = image.buffer();

        let builder = std::thread::Builder::new()
            .name(format!("render:{}", i))
            .stack_size(32 * 1024 * 1024);

        builder
            .spawn(move || loop {
                for y in 0..image.height {
                    for x in 0..image.width {
                        let u = (x as f64 + f64::rand()) / ((image.width - 1) as f64);
                        let v = (y as f64 + f64::rand()) / ((image.height - 1) as f64);
                        let ray = camera.ray(u, v);
                        let (color, depth) = camera.trace(&*world, ray, MAX_DEPTH);

                        buffer.set((x, y), color, MAX_DEPTH - depth);
                    }
                }

                image.merge(&mut buffer);
                event_proxy
                    .lock()
                    .expect("Event proxy posioned")
                    .send_event(UserEvent::Update)
                    .expect("Unable to reach event loop");
            })
            .expect("Unable to spawn render thread");
    }
}

fn run(event_loop: EventLoop<UserEvent>, image: Arc<Image>) -> ! {
    let window_size = PhysicalSize::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    let window_builder = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_title("Mass Raytrace");

    let context_builder = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_srgb(true)
        .with_gl_profile(glutin::GlProfile::Core)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 2)));

    let display = glium::Display::new(window_builder, context_builder, &event_loop)
        .expect("Unable to create display");
    let program = Program::from_source(&display, VERTEX_SRC, FRAGMENT_SRC, None)
        .expect("Unable to create gl program");
    let vertex_buffer =
        glium::VertexBuffer::new(&display, &QUAD).expect("Unable to create vertex buffer");

    let mut display_depth = false;

    event_loop.run(move |event, _window, control_flow| match event {
        Event::UserEvent(UserEvent::Update) => {
            display.gl_window().window().request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state: winit::event::ElementState::Released,
                            ..
                        },
                    ..
                },
            ..
        } => match key {
            VirtualKeyCode::E => {
                let path = format!(
                    "./export/raytrace_{}.png",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_else(|e| e.duration())
                        .as_secs()
                );
                image.dump(&path);
                println!("Image saved to: {}", path);
            }
            VirtualKeyCode::D => {
                display_depth = !display_depth;
                display.gl_window().window().request_redraw();
            }
            _ => (),
        },
        Event::RedrawRequested(_) => {
            let tex = image.fill(&display, display_depth);
            let mut frame = display.draw();

            frame.clear_color(0.0, 0.0, 0.0, 1.0);

            let uniforms = uniform! {
                quad_texture: tex.sampled()
            };

            frame
                .draw(
                    &vertex_buffer,
                    glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
                    &program,
                    &uniforms,
                    &DrawParameters::default(),
                )
                .expect("Unable to draw frame");

            frame.finish().expect("Unable to finish frame");
        }
        _ => (),
    });
}

struct ImageBuffer {
    pixels: Vec<((f32, f32, f32), u32)>,
    width: u32,
    height: u32,
}

impl ImageBuffer {
    fn new(width: u32, height: u32) -> Self {
        ImageBuffer {
            pixels: vec![((0.0, 0.0, 0.0), 0); (width * height) as usize],
            width,
            height,
        }
    }

    fn set(&mut self, position: (u32, u32), color: V3, depth: i32) {
        let index = ((position.1 * self.width) + position.0) as usize;
        self.pixels[index] = (
            (color.x() as f32, color.y() as f32, color.z() as f32),
            depth.max(0) as u32,
        );
    }
}

struct Image {
    pixels: std::sync::Mutex<(u32, Vec<(V3, u32)>)>,
    width: u32,
    height: u32,
}

impl Image {
    fn new(width: u32, height: u32) -> Self {
        Image {
            pixels: std::sync::Mutex::new((0, vec![(V3::fill(0.0), 0); (width * height) as usize])),
            width,
            height,
        }
    }

    fn buffer(&self) -> ImageBuffer {
        ImageBuffer::new(self.width, self.height)
    }

    fn merge(&self, buffer: &ImageBuffer) {
        let mut pixels = self.pixels.lock().unwrap();
        for (&(buf_color, buf_depth), (image_color, image_depth)) in
            buffer.pixels.iter().zip(pixels.1.iter_mut())
        {
            *image_color += V3::new(buf_color.0 as f64, buf_color.1 as f64, buf_color.2 as f64);
            *image_depth = (*image_depth).max(buf_depth);
        }
        pixels.0 += 1;
    }

    fn to_rgba_bytes(&self, show_depth: bool) -> Vec<u8> {
        let pixels = self.pixels.lock().unwrap();
        if pixels.0 == 0 {
            let mut pixel_bytes = Vec::with_capacity(pixels.1.len() * 4);
            for _ in 0..pixels.1.len() {
                pixel_bytes.push(0);
                pixel_bytes.push(0);
                pixel_bytes.push(0);
                pixel_bytes.push(255);
            }
            pixel_bytes
        } else {
            let mut pixel_bytes = Vec::with_capacity(pixels.1.len() * 4);
            let scale = 1.0 / pixels.0 as f64;
            let component = |f_c: f64| ((scale * f_c).sqrt().min(1.0).max(0.0) * 255.0) as u8;

            if show_depth {
                let max_depth = pixels.1.iter().map(|p| p.1).max().unwrap_or(1).max(1);
                for (_color, depth) in pixels.1.iter() {
                    let depth = ((*depth as f64 / max_depth as f64) * 255.0) as u8;
                    pixel_bytes.push(depth);
                    pixel_bytes.push(depth);
                    pixel_bytes.push(depth);
                    pixel_bytes.push(255);
                }
            } else {
                for (color, _depth) in pixels.1.iter() {
                    pixel_bytes.push(component(color.x()));
                    pixel_bytes.push(component(color.y()));
                    pixel_bytes.push(component(color.z()));
                    pixel_bytes.push(255);
                }
            }
            pixel_bytes
        }
    }

    fn fill<F: Facade>(&self, display: &F, show_depth: bool) -> SrgbTexture2d {
        let pixel_bytes = self.to_rgba_bytes(show_depth);

        let data = glium::texture::RawImage2d {
            data: pixel_bytes.into(),
            width: self.width as u32,
            height: self.height as u32,
            format: glium::texture::ClientFormat::U8U8U8U8,
        };
        let texture = SrgbTexture2d::new(display, data).expect("Unable to create texture");

        texture
    }

    fn dump<P: AsRef<std::path::Path>>(&self, path: P) {
        let path = path.as_ref();
        let pixel_bytes = self.to_rgba_bytes(false);
        let pixel_bytes: Vec<u8> = pixel_bytes
            .chunks(4 * self.width as usize)
            .rev()
            .flat_map(|c| c)
            .map(|p| *p)
            .collect();

        std::fs::create_dir_all(&path.parent().expect("input path should have parent"))
            .expect("Unable to create export directory");
        let r = image::save_buffer_with_format(
            path,
            &pixel_bytes,
            self.width,
            self.height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        );
        if let Err(error) = r {
            eprintln!("Unable to save image: {:?}", error);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: (f32, f32),
    uv: (f32, f32),
}

implement_vertex!(Vertex, position, uv);

const QUAD: [Vertex; 4] = [
    Vertex {
        position: (-1.0, -1.0),
        uv: (0.0, 0.0),
    },
    Vertex {
        position: (-1.0, 1.0),
        uv: (0.0, 1.0),
    },
    Vertex {
        position: (1.0, -1.0),
        uv: (1.0, 0.0),
    },
    Vertex {
        position: (1.0, 1.0),
        uv: (1.0, 1.0),
    },
];

const VERTEX_SRC: &'static str = "
#version 420

in vec2 position;
in vec2 uv;

out vec2 v_uv;

void main() {
   v_uv = uv;
   gl_Position = vec4(position.x, position.y, 1.0, 1.0);
}";

const FRAGMENT_SRC: &'static str = "
#version 420

in vec2 v_uv;

out vec4 f_color;

uniform sampler2D quad_texture;

void main () {
   f_color = texture(quad_texture, v_uv);
}";
