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

use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
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
    Redraw(Vec<u8>),
    FatalError,
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

static PIXEL_UPDATE_FLAG: AtomicBool = AtomicBool::new(false);

fn main() {
    let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event();
    let event_proxy = Arc::new(Mutex::new(event_loop.create_proxy()));
    let image = Arc::new(Image::new(IMAGE_WIDTH, IMAGE_HEIGHT));

    {
        let image = image.clone();
        std::thread::spawn(move || {
            let res = std::panic::catch_unwind(|| worker(image, event_proxy.clone()));
            match res {
                Err(_err) => event_proxy
                    .lock()
                    .expect("event proxy poisioned")
                    .send_event(UserEvent::FatalError)
                    .expect("event loop disconnected"),
                _ => (),
            }
        });
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
        //let (mut world, camera) = scenes::mario(animation_t, ASPECT_RATIO);
        //let (mut world, camera) = scenes::sphere_grid(animation_t, ASPECT_RATIO);
        //let (mut world, camera) = scenes::scratchpad(animation_t, ASPECT_RATIO);
        //let (mut world, camera) = scenes::lucy(animation_t, ASPECT_RATIO);

        world.build_bvh();

        {
            let image = image.clone();
            let event_proxy = event_proxy.clone();
            render(image, event_proxy, world, camera, samples_per_frame);
        }

        frame += 1;
        if ANIMATING {
            image.dump(
                format!("animation/frame_{:05}.png", frame),
                DisplayMode::Default,
            );

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

    let mut albedo_buf = FloatBuffer::new(image.width, image.height);
    let mut normal_buf = FloatBuffer::new(image.width, image.height);

    for y in 0..image.height {
        for x in 0..image.width {
            let u = (x as f32) / ((image.width - 1) as f32);
            let v = (y as f32) / ((image.height - 1) as f32);
            let ray = camera.ray(u, v);
            let (albedo, normal) = camera.albedo_normal(&*world, ray);

            albedo_buf.set((x, y), albedo);
            normal_buf.set((x, y), normal);
        }
    }

    image.set_albedo(albedo_buf);
    image.set_normal(normal_buf);

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

    let mut display_mode = DisplayMode::Default;
    let event_proxy = event_loop.create_proxy();

    let mut texture = None;

    event_loop.run(move |event, _window, control_flow| match event {
        Event::UserEvent(UserEvent::Update) => {
            let image = image.clone();
            let event_proxy = event_proxy.clone();
            std::thread::spawn(move || {
                if let Ok(_) = PIXEL_UPDATE_FLAG.compare_exchange(
                    false,
                    true,
                    AtomicOrdering::Acquire,
                    AtomicOrdering::Relaxed,
                ) {
                    let image_bytes = image.to_rgb_bytes(display_mode);
                    if let Err(err) = event_proxy.send_event(UserEvent::Redraw(image_bytes)) {
                        eprintln!("{}", err);
                    }
                    PIXEL_UPDATE_FLAG.store(false, AtomicOrdering::Release);
                }
            });
        }
        Event::UserEvent(UserEvent::Redraw(frame)) => {
            let data = glium::texture::RawImage2d {
                data: frame.into(),
                width: image.width as u32,
                height: image.height as u32,
                format: glium::texture::ClientFormat::U8U8U8,
            };
            texture = Some(SrgbTexture2d::new(&display, data).expect("Unable to create texture"));
            display.gl_window().window().request_redraw();
        }
        Event::UserEvent(UserEvent::FatalError) => {
            eprintln!("Render thread panic");
            *control_flow = ControlFlow::Exit;
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
                image.dump(&path, display_mode);
                println!("Image saved to: {}", path);
            }
            VirtualKeyCode::Key1 => {
                display_mode = DisplayMode::Default;
                event_proxy
                    .send_event(UserEvent::Update)
                    .expect("Unable to reach event loop");
            }
            VirtualKeyCode::Key2 => {
                display_mode = DisplayMode::Denoise;
                event_proxy
                    .send_event(UserEvent::Update)
                    .expect("Unable to reach event loop");
            }
            VirtualKeyCode::Key3 => {
                display_mode = DisplayMode::Depth;
                event_proxy
                    .send_event(UserEvent::Update)
                    .expect("Unable to reach event loop");
            }
            VirtualKeyCode::Key4 => {
                display_mode = DisplayMode::Albedo;
                event_proxy
                    .send_event(UserEvent::Update)
                    .expect("Unable to reach event loop");
            }
            VirtualKeyCode::Key5 => {
                display_mode = DisplayMode::Normal;
                event_proxy
                    .send_event(UserEvent::Update)
                    .expect("Unable to reach event loop");
            }
            _ => (),
        },
        Event::RedrawRequested(_) => {
            if let Some(texture) = texture.as_ref() {
                let mut frame = display.draw();

                frame.clear_color(0.0, 0.0, 0.0, 1.0);

                let uniforms = uniform! {
                    quad_texture: texture.sampled()
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
        }
        _ => (),
    });
}

#[derive(Copy, Clone, PartialEq)]
enum DisplayMode {
    Default,
    Denoise,
    Depth,
    Albedo,
    Normal,
}

struct FloatBuffer {
    pixels: Vec<f32>,
    width: u32,
    height: u32,
}

impl FloatBuffer {
    fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![0.0; (width * height) as usize * 3],
            width,
            height,
        }
    }

    fn set(&mut self, position: (u32, u32), color: V3) {
        let index = ((position.1 * self.width * 3) + (position.0 * 3)) as usize;
        self.pixels[index + 0] = color.x();
        self.pixels[index + 1] = color.y();
        self.pixels[index + 2] = color.z();
    }

    fn as_slice(&self) -> &[f32] {
        &*self.pixels
    }
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
    pixels: Mutex<(u32, Vec<(V3, u32)>)>,
    width: u32,
    height: u32,
    albedo: Mutex<Option<FloatBuffer>>,
    normal: Mutex<Option<FloatBuffer>>,
}

impl Image {
    fn new(width: u32, height: u32) -> Self {
        Image {
            pixels: Mutex::new((0, vec![(V3::zero(), 0); (width * height) as usize])),
            width,
            height,
            albedo: Mutex::new(None),
            normal: Mutex::new(None),
        }
    }

    fn set_albedo(&self, albedo: FloatBuffer) {
        *self.albedo.lock().unwrap() = Some(albedo);
    }

    fn set_normal(&self, normal: FloatBuffer) {
        *self.normal.lock().unwrap() = Some(normal);
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

    fn to_rgb_bytes(&self, mode: DisplayMode) -> Vec<u8> {
        let pixels = self.pixels.lock().unwrap();
        if pixels.0 == 0 {
            let mut pixel_bytes = Vec::with_capacity(pixels.1.len() * 3);
            for _ in 0..pixels.1.len() {
                pixel_bytes.push(0);
                pixel_bytes.push(0);
                pixel_bytes.push(0);
            }
            pixel_bytes
        } else {
            let scale = 1.0 / pixels.0 as f32;
            let component = |f_c: f32| ((scale * f_c).powf(1.0 / 2.2).min(1.0).max(0.0));
            let mut pixel_floats = Vec::with_capacity(pixels.1.len() * 3);

            let pixel_floats = match mode {
                DisplayMode::Depth => {
                    let max_depth = pixels.1.iter().map(|p| p.1).max().unwrap_or(1).max(1);
                    let max_depth = max_depth as f32 * scale;
                    for (_color, depth) in pixels.1.iter() {
                        let depth =
                            (((*depth as f32 * scale) / max_depth).max(0.0).min(1.0)) as f32;
                        pixel_floats.push(depth);
                        pixel_floats.push(depth);
                        pixel_floats.push(depth);
                    }

                    pixel_floats
                }
                DisplayMode::Default => {
                    for (color, _depth) in pixels.1.iter() {
                        pixel_floats.push(component(color.x()));
                        pixel_floats.push(component(color.y()));
                        pixel_floats.push(component(color.z()));
                    }
                    pixel_floats
                }
                DisplayMode::Denoise => {
                    for (color, _depth) in pixels.1.iter() {
                        pixel_floats.push(component(color.x()));
                        pixel_floats.push(component(color.y()));
                        pixel_floats.push(component(color.z()));
                    }
                    self.denoise(&mut pixel_floats);
                    pixel_floats
                }
                DisplayMode::Albedo => {
                    let albedo = self.albedo.lock();
                    if let Ok(Some(albedo)) = albedo.as_deref() {
                        for p in albedo.as_slice() {
                            pixel_floats.push(p.min(1.0).max(0.0).powf(1.0 / 2.2));
                        }
                    } else {
                        for _ in 0..pixels.1.len() {
                            pixel_floats.push(0.0);
                            pixel_floats.push(0.0);
                            pixel_floats.push(0.0);
                        }
                    }

                    pixel_floats
                }
                DisplayMode::Normal => {
                    let normal = self.normal.lock();
                    if let Ok(Some(normal)) = normal.as_deref() {
                        for p in normal.as_slice() {
                            pixel_floats.push((p + 1.0) / 2.0);
                        }
                    } else {
                        for _ in 0..pixels.1.len() {
                            pixel_floats.push(0.0);
                            pixel_floats.push(0.0);
                            pixel_floats.push(0.0);
                        }
                    }

                    pixel_floats
                }
            };

            pixel_floats
                .into_iter()
                .map(|p| (p * 255.0) as u8)
                .collect()
        }
    }

    #[cfg(feature = "denoise")]
    fn denoise(&self, pixels: &mut [f32]) {
        let albedo = self.albedo.lock();
        let normal = self.normal.lock();
        let device = oidn::Device::new();
        if let (Ok(Some(albedo)), Ok(Some(normal))) = (albedo.as_deref(), normal.as_deref()) {
            oidn::RayTracing::new(&device)
                .image_dimensions(self.width as usize, self.height as usize)
                .srgb(true)
                .clean_aux(false)
                .albedo_normal(albedo.as_slice(), normal.as_slice())
                .filter_in_place(pixels)
                .expect("unable to denoise filter");
        } else {
            oidn::RayTracing::new(&device)
                .image_dimensions(self.width as usize, self.height as usize)
                .srgb(true)
                .filter_in_place(pixels)
                .expect("unable to denoise filter");
        };
    }

    #[cfg(not(feature = "denoise"))]
    fn denoise(&self, pixels: &mut [f32]) {}

    fn clear(&self) {
        let mut pixels = self.pixels.lock().unwrap();

        for (pixel, depth) in pixels.1.iter_mut() {
            *pixel = V3::zero();
            *depth = 0;
        }

        pixels.0 = 0;
    }

    fn dump<P: AsRef<std::path::Path>>(&self, path: P, mode: DisplayMode) {
        let path = path.as_ref();
        let pixel_bytes = self.to_rgb_bytes(mode);
        let pixel_bytes: Vec<u8> = pixel_bytes
            .chunks(3 * self.width as usize)
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
            image::ColorType::Rgb8,
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
