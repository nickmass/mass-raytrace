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

mod eve;
mod geom;
mod material;
mod obj_loader;
mod ply_loader;
mod scenes;
mod stl_loader;
mod texture;
mod world;

#[derive(Debug)]
enum UserEvent {
    Update,
    Complete,
}

const MAX_DEPTH: u32 = 150;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const IMAGE_WIDTH: u32 = 1920 * 2;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32;

const ANIMATING: bool = false;
const FRAMES_PER_SECOND: u32 = 24;
const ANIMATION_DURATION: u32 = 5;
const TOTAL_FRAMES: u32 = FRAMES_PER_SECOND * ANIMATION_DURATION;
const SAMPLES_PER_FRAME: u32 = 10;

fn main() {
    let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event();
    let event_proxy = Arc::new(Mutex::new(event_loop.create_proxy()));
    let image = Arc::new(Image::new(IMAGE_WIDTH, IMAGE_HEIGHT));

    {
        let image = image.clone();
        std::thread::spawn(|| worker(image, event_proxy));
    }

    run(event_loop, image)
}

fn worker(image: Arc<Image>, event_proxy: Arc<Mutex<EventLoopProxy<UserEvent>>>) {
    fastrand::seed(1);

    let mut frame = 0;
    let samples_per_frame = if ANIMATING {
        Some(SAMPLES_PER_FRAME)
    } else {
        None
    };

    let start_time = std::time::Instant::now();
    while frame < TOTAL_FRAMES {
        image.clear();

        let animation_t = frame as f32 / TOTAL_FRAMES as f32;

        let (mut world, camera) = scenes::cornell_box(animation_t, ASPECT_RATIO);

        world.build_bvh();

        {
            let image = image.clone();
            let event_proxy = event_proxy.clone();
            render(image, event_proxy, world, camera, samples_per_frame);
        }

        frame += 1;
        if ANIMATING {
            image.dump(format!("animation/frame_{:05}.png", frame));

            let elapsed_s = start_time.elapsed().as_secs() as f32;
            let complete = frame as f32 / TOTAL_FRAMES as f32;
            let total_s = (1.0 / complete) * elapsed_s;
            let remaining_s = total_s - elapsed_s;

            println!(
                "Animation: {:.2}%  ~{:.1} minutes remaining, {:.1} minutes elapsed.",
                complete * 100.0,
                remaining_s / 60.0,
                elapsed_s / 60.0
            );
        }
    }

    event_proxy
        .lock()
        .expect("Event proxy posioned")
        .send_event(UserEvent::Complete)
        .expect("Unable to reach event loop");
}

fn render<B: 'static + material::Background>(
    image: Arc<Image>,
    event_proxy: Arc<Mutex<EventLoopProxy<UserEvent>>>,
    world: world::World<B>,
    camera: world::Camera,
    frame_limit: Option<u32>,
) {
    let world = Arc::new(world);
    let camera = Arc::new(camera);
    let cpus = num_cpus::get() as i32;
    let cpus = (cpus - 2).max(1);

    let mut handles = Vec::new();
    for i in 0..cpus {
        let event_proxy = event_proxy.clone();
        let world = world.clone();
        let camera = camera.clone();
        let image = image.clone();
        let mut buffer = image.buffer();
        let mut first = true;

        let mut frame_limit = frame_limit.clone();

        let builder = std::thread::Builder::new()
            .name(format!("render:{}", i))
            .stack_size(32 * 1024 * 1024);

        let handle = builder
            .spawn(move || {
                while frame_limit.is_none() || frame_limit != Some(0) {
                    let frame_start = std::time::Instant::now();
                    for y in 0..image.height {
                        if i == 0 && first && frame_limit.is_none() && y % 10 == 0 {
                            println!("{:.2}%", y as f64 / image.height as f64 * 100.0);
                        }
                        for x in 0..image.width {
                            let u = (x as f32 + f32::rand()) / ((image.width - 1) as f32);
                            let v = (y as f32 + f32::rand()) / ((image.height - 1) as f32);
                            let ray = camera.ray(u, v);
                            let (color, depth) = camera.trace(&*world, ray, MAX_DEPTH);

                            buffer.set((x, y), color, MAX_DEPTH - depth);
                        }
                    }

                    first = false;

                    if frame_limit.is_none() || i == 0 {
                        println!("Frame time: {} seconds", frame_start.elapsed().as_secs());
                    }

                    image.merge(&mut buffer);
                    event_proxy
                        .lock()
                        .expect("Event proxy posioned")
                        .send_event(UserEvent::Update)
                        .expect("Unable to reach event loop");

                    frame_limit.as_mut().map(|n| *n -= 1);
                }
            })
            .expect("Unable to spawn render thread");

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
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
    pixels: Vec<(V3, u32)>,
    width: u32,
    height: u32,
}

impl ImageBuffer {
    fn new(width: u32, height: u32) -> Self {
        ImageBuffer {
            pixels: vec![(V3::zero(), 0); (width * height) as usize],
            width,
            height,
        }
    }

    fn set(&mut self, position: (u32, u32), color: V3, depth: u32) {
        let index = ((position.1 * self.width) + position.0) as usize;
        self.pixels[index] = (color, depth);
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
            pixels: std::sync::Mutex::new((0, vec![(V3::zero(), 0); (width * height) as usize])),
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
            *image_color += buf_color;
            *image_depth += buf_depth;
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
            let scale = 1.0 / pixels.0 as f32;
            let component = |f_c: f32| ((scale * f_c).sqrt().min(1.0).max(0.0) * 255.0) as u8;

            if show_depth {
                let max_depth = pixels.1.iter().map(|p| p.1).max().unwrap_or(1).max(1);
                let max_depth = max_depth as f32 * scale;
                for (_color, depth) in pixels.1.iter() {
                    let depth =
                        (((*depth as f32 * scale) / max_depth).max(0.0).min(1.0) * 255.0) as u8;
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

    fn clear(&self) {
        let mut pixels = self.pixels.lock().unwrap();

        for (pixel, depth) in pixels.1.iter_mut() {
            *pixel = V3::zero();
            *depth = 0;
        }

        pixels.0 = 0;
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
