#[macro_use]
pub extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate winit;
extern crate alga;
extern crate nalgebra;
extern crate rand;
extern crate noise;
extern crate image;
extern crate time;
extern crate rayon;

pub mod scene;
mod camera;
mod texture;
mod icosphere;

pub use scene::Scene;
pub use camera::Camera;
pub use texture::generate as generate_texture;
pub use icosphere::generate as generate_icosphere;

use std::env;
use std::f32::consts::PI;
use nalgebra::{Vector2, Vector3, UnitQuaternion};
use time::precise_time_s;
use gfx::Device;
use scene::{ColorFormat, DepthFormat};

pub fn run<I>(title: &str)
    where I: scene::Impl<gfx_device_gl::Resources>
{
    let mut args = env::args().skip(1);

    let sphere_count: usize = args.next()
        .map(|s| s.parse().expect("expected number of spheres")).unwrap_or(64);
    let texture_size: usize = args.next()
        .map(|s| s.parse().expect("expected texture size")).unwrap_or(128);

    let gl_version = glutin::GlRequest::GlThenGles {
        opengl_version: (3, 2),
        opengles_version: (2, 0)
    };
    let wb = glutin::WindowBuilder::new()
        .with_gl(gl_version)
        .with_title(format!("Primus Polygoni -- {}", title));
    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(wb);
    let (width, height) = window.get_inner_size_points().unwrap();
    let mut aspect_ratio = width as f32 / height as f32;

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let mut scene = Scene::<_, I>::new(sphere_count,
                                       texture_size,
                                       main_color,
                                       main_depth,
                                       &mut factory,
                                       &mut encoder);

    let mut mouse = Vector2::new(0., 0.);
    let mut head_spinning = false;
    let mut going_left = false;
    let mut going_right = false;
    let mut toggled = false;
    let mut reset = true;
    let mut update_marker = None;

    let mut marker = precise_time_s() as f32;
    let mut fps_counter = FpsCounter::new(1.0);
    'main: loop {
        for event in window.poll_events() {
            use glutin::Event::*;
            use glutin::ElementState::*;
            use glutin::MouseButton::*;
            match event {
                Closed => break 'main,
                Resized(w, h) => {
                    aspect_ratio = w as f32 / h as f32;
                    gfx_window_glutin::update_views(
                        &window,
                        &mut scene.data.color_target,
                        &mut scene.data.depth_target
                    );
                }
                KeyboardInput(state, _, Some(key)) => {
                    use glutin::VirtualKeyCode::*;
                    match key {
                        Q | Left => going_left = state == Pressed,
                        D | Right => going_right = state == Pressed,
                        T if state == Released => toggled = !toggled,
                        R if state == Released => reset = true,
                        _ => {}
                    }
                },
                MouseMoved(x, y) => {
                    let (w, h, _, _) = scene.data.color_target.get_dimensions();
                    mouse = Vector2::new((x as f32 / w as f32) - 0.5,
                                         0.5 - (y as f32 / h as f32));
                }
                MouseInput(state, Left) => head_spinning = state == Pressed,
                _ => {}
            }
        }

        let now = precise_time_s() as f32;
        let delta = now - marker;
        update_marker.take().map(|um| {
            println!("update time: {} ms", (now - um) * 1_000.);
        });
        fps_counter.update(delta).map(|fps| println!("{} fps", fps));
        marker = now;

        if head_spinning {
            let max_rotation = 2.0 * PI * delta;
            scene.camera.rotate(UnitQuaternion::new(
                Vector3::y() * (-mouse.x * max_rotation)
            ));
            scene.camera.pitch(mouse.y * max_rotation);
        }

        let speed = 0.5 * PI;
        if going_left { scene.camera.move_left(speed * delta); }
        if going_right { scene.camera.move_right(speed * delta); }
        if toggled { unimplemented!() }
        if reset {
            print!("generating textures ... ");
            let before = precise_time_s() as f32;
            scene.generate_textures(&mut encoder, &mut factory);
            let after = precise_time_s() as f32;
            reset = false;
            println!("took {} ms", (after - before) * 1_000.);
            update_marker = Some(after);
        }

        encoder.clear(&scene.data.color_target, [0.1, 0.2, 0.3, 1.0]);
        encoder.clear_depth(&scene.data.depth_target, 1.0);
        scene.render(aspect_ratio, &mut encoder);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

pub struct FpsCounter {
    accumulator: f32,
    frames: f32,
    period: f32,
}

impl FpsCounter {
    pub fn new(sample_period: f32) -> Self {
        FpsCounter {
            accumulator: 0.,
            frames: 0.,
            period: sample_period,
        }
    }

    pub fn update(&mut self, delta: f32) -> Option<f32> {
        self.accumulator += delta;
        self.frames += 1.;

        if self.accumulator >= self.period {
            let fps = self.frames / self.accumulator;
            self.accumulator = 0.;
            self.frames = 0.;
            Some(fps)
        } else {
            None
        }
    }
}
