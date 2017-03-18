extern crate primus_polygoni;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;
extern crate time;

use std::f32::consts::PI;
use primus_polygoni::rand;
use primus_polygoni::rand::distributions::{IndependentSample, Range};
use primus_polygoni::gfx;
use primus_polygoni::nalgebra::{self, Vector2, Vector3, UnitQuaternion};
use time::precise_time_s;

use primus_polygoni::{Vertex, Instance, Locals, Camera, ColorFormat, DepthFormat};
use gfx::traits::FactoryExt;
use gfx::{Factory, Device};

const SCENE_SPHERES: usize = 100;
const SCENE_RADIUS: f32 = (SCENE_SPHERES as f32 * 4.0) / (2.0 * PI);

fn fill_instances<R, C>(encoder: &mut gfx::Encoder<R, C>, 
                        instances: &gfx::handle::Buffer<R, Instance>)
    where R: gfx::Resources, C: gfx::CommandBuffer<R> 
{
    let mut vec = Vec::with_capacity(SCENE_SPHERES);
    let mut rng = rand::thread_rng();

    for i in 0..SCENE_SPHERES {
        let angle = i as f32 / SCENE_SPHERES as f32 * (2.0 * PI);
        let position = Vector3::new(
            angle.cos() * SCENE_RADIUS,
            0.0,
            angle.sin() * SCENE_RADIUS);

        let radius = Range::new(0.5, 1.5).ind_sample(&mut rng);
        let remaining = 2.0 - radius;

        let range = Range::new(-remaining, remaining);
        let displacement = Vector3::new(
            range.ind_sample(&mut rng),
            range.ind_sample(&mut rng),
            range.ind_sample(&mut rng),
        );

        let transform = nalgebra::Similarity3::from_parts(
            nalgebra::Translation3::from_vector(position + displacement),
            nalgebra::one(),
            radius).to_homogeneous();
        let transform = transform.as_slice();

        let line = |l: &[f32]| [l[0], l[1], l[2], l[3]]; 

        vec.push(Instance {
            t1: line(&transform[0..4]),
            t2: line(&transform[4..8]),
            t3: line(&transform[8..12]),
            t4: line(&transform[12..16]),
        });
    }
    encoder.update_buffer(instances, &vec[..], 0).unwrap();
}

fn main() {
    let gl_version = glutin::GlRequest::GlThenGles {
        opengl_version: (3, 2),
        opengles_version: (2, 0)
    };
    let wb = glutin::WindowBuilder::new()
        .with_gl(gl_version)
        .with_title("Primus Polygoni");
    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(wb);
    let (width, height) = window.get_inner_size_points().unwrap();
    let mut aspect_ratio = width as f32 / height as f32;

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = primus_polygoni::create_pipeline(&mut factory);

    let (vertex_data, index_data) = primus_polygoni::generate_icosphere(4);
    let (vertices, mut slice) = factory
        .create_vertex_buffer_with_slice(&vertex_data[..], &index_data[..]);
    slice.instances = Some((SCENE_SPHERES as u32, 0));

    let instances = factory.create_buffer(SCENE_SPHERES,
                                          gfx::buffer::Role::Vertex,
                                          gfx::memory::Usage::Dynamic,
                                          gfx::Bind::empty()).unwrap();
    fill_instances(&mut encoder, &instances);

    let size = 256;
    let (w, h) = (size * 2, size);
   
    let mut texels: Vec<_> = (0..(w * h * SCENE_SPHERES)).map(|_| [0; 4]).collect();
    let mut textures = Vec::with_capacity(SCENE_SPHERES);

    for s in texels.chunks_mut(w *h) {  
        primus_polygoni::generate_texture(s, size);
        textures.push(s as &[_]);
    }

    let (_, texture_view) =
        factory.create_texture_immutable::<gfx::format::Rgba8>(
            gfx::texture::Kind::D2Array(w as gfx::texture::Size,
                                        h as gfx::texture::Size,
                                        SCENE_SPHERES as gfx::texture::Size,
                                        gfx::texture::AaMode::Single),
            &textures[..]
        ).expect("could not create texture");

    let sinfo = gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Clamp);

    let mut data = primus_polygoni::pipe::Data {
        vertices: vertices,
        instances: instances,
        locals: factory.create_constant_buffer(1),
        color: (texture_view, factory.create_sampler(sinfo)),
        color_target: main_color,
        depth_target: main_depth,
    };

    let mut camera = Camera::new(SCENE_RADIUS);

    let mut mouse = Vector2::new(0., 0.);
    let mut head_spinning = false;
    let mut going_left = false;
    let mut going_right = false;

    let mut marker = precise_time_s() as f32;
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
                        &window, &mut data.color_target, &mut data.depth_target);
                }
                KeyboardInput(state, _, Some(key)) => {
                    use glutin::VirtualKeyCode::*;
                    match key {
                        Q | Left => going_left = state == Pressed,
                        D | Right => going_right = state == Pressed,
                        _ => {}
                    }
                },
                MouseMoved(x, y) => {
                    let (w, h, _, _) = data.color_target.get_dimensions();
                    mouse = Vector2::new((x as f32 / w as f32) - 0.5,
                                         0.5 - (y as f32 / h as f32));
                }
                MouseInput(state, Left) => head_spinning = state == Pressed,
                _ => {}
            }
        }

        let now = precise_time_s() as f32;
        let delta = now - marker;
        marker = now;

        if head_spinning {
            let max_rotation = 2.0 * PI * delta;
            camera.rotate(UnitQuaternion::new(
                Vector3::y() * (-mouse.x * max_rotation)
            ));
            camera.pitch(mouse.y * max_rotation);
        }
        
        let speed = 0.5 * PI;
        if going_left { camera.move_left(speed * delta); }
        if going_right { camera.move_right(speed * delta); }

        camera.update(aspect_ratio);
        encoder.update_constant_buffer(&data.locals, &Locals {
            transform: camera.gpu_transform()
        });

        encoder.clear(&data.color_target, [0.1, 0.2, 0.3, 1.0]);
        encoder.clear_depth(&data.depth_target, 1.0);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
