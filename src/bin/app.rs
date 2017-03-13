extern crate primus_polygoni;
extern crate image;
extern crate time;

use std::f32::consts::PI;
use primus_polygoni::rand;
use primus_polygoni::rand::distributions::{IndependentSample, Range};

use primus_polygoni::gfx;
use primus_polygoni::gfx_app;
use primus_polygoni::winit;
use primus_polygoni::nalgebra::{self, Vector2, Vector3, UnitQuaternion};
use time::precise_time_s;

use primus_polygoni::{Vertex, Instance, Locals, Camera};

const SCENE_SPHERES: usize = 100;
const SCENE_RADIUS: f32 = (SCENE_SPHERES as f32 * 4.0) / (2.0 * PI);

struct App<R: gfx::Resources> {
    bundle: gfx::Bundle<R, primus_polygoni::pipe::Data<R>>,
    camera: Camera,
    aspect_ratio: f32,
    mouse: Vector2<f32>,
    head_spinning: bool,
    going_left: bool,
    going_right: bool,
    marker: f32,
    init: bool,
}

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

impl<R: gfx::Resources> gfx_app::Application<R> for App<R> {
    fn new<F: gfx::Factory<R>>(factory: &mut F,
                               backend: gfx_app::shade::Backend,
                               targets: gfx_app::WindowTargets<R>) -> Self {
        use gfx::traits::FactoryExt;

        let pso = primus_polygoni::create_pipeline(factory, backend);

        let (vertex_data, index_data) = primus_polygoni::generate_icosphere(4);
        let (vertices, mut slice) = factory
            .create_vertex_buffer_with_slice(&vertex_data[..], &index_data[..]);
        slice.instances = Some((SCENE_SPHERES as u32, 0));

        let size = 1024;
        let (w, h) = (size * 2, size);
        let mut texels: Vec<_> = (0..(w * h)).map(|_| [0; 4]).collect();
        primus_polygoni::generate_texture(&mut texels, size);

        let (_, texture_view) =
            factory.create_texture_immutable::<gfx::format::Rgba8>(
                gfx::texture::Kind::D2(w as gfx::texture::Size, 
                                       h as gfx::texture::Size, 
                                       gfx::texture::AaMode::Single),
                &[&texels]
            ).unwrap();
        
        let sinfo = gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Bilinear,
            gfx::texture::WrapMode::Clamp);

        let data = primus_polygoni::pipe::Data {
            vertices: vertices,
            instances: factory.create_buffer(SCENE_SPHERES, gfx::buffer::Role::Vertex,
                                             gfx::memory::Usage::Dynamic, gfx::Bind::empty()).unwrap(),
            locals: factory.create_constant_buffer(1),
            color: (texture_view, factory.create_sampler(sinfo)),
            color_target: targets.color,
            depth_target: targets.depth,
        };

        App {
            bundle: gfx::Bundle::new(slice, pso, data),
            camera: Camera::new(SCENE_RADIUS),
            aspect_ratio: targets.aspect_ratio,
            mouse: Vector2::new(0., 0.),
            head_spinning: false,
            going_left: false,
            going_right: false,
            marker: precise_time_s() as f32,
            init: false,
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
        
        let now = precise_time_s() as f32;
        let delta = now - self.marker;
        self.marker = now;

        if !self.init {
            fill_instances(encoder, &self.bundle.data.instances);
            self.init = true;
        }

        if self.head_spinning {
            let max_rotation = 2.0 * PI * delta;
            self.camera.rotate(UnitQuaternion::new(
                Vector3::y() * (-self.mouse.x * max_rotation)
            ));
            self.camera.pitch(self.mouse.y * max_rotation);
        }
        
        let speed = 0.5 * PI;
        if self.going_left { self.camera.move_left(speed * delta); }
        if self.going_right { self.camera.move_right(speed * delta); }

        self.camera.update(self.aspect_ratio);
        let locals = Locals { 
            transform: self.camera.gpu_transform()
        };
        encoder.update_constant_buffer(&self.bundle.data.locals, &locals);

        encoder.clear(&self.bundle.data.color_target, [0.1, 0.2, 0.3, 1.0]);
        encoder.clear_depth(&self.bundle.data.depth_target, 1.0);
        self.bundle.encode(encoder);
    }

    fn on_resize(&mut self, targets: gfx_app::WindowTargets<R>) {
        self.bundle.data.color_target = targets.color;
        self.bundle.data.depth_target = targets.depth;
        self.aspect_ratio = targets.aspect_ratio;
    }

    fn on(&mut self, event: winit::Event) {
        use winit::Event::*;
        use winit::ElementState::*;
        use winit::MouseButton::*;
        match event {
            KeyboardInput(state, _, Some(key)) => {
                use winit::VirtualKeyCode::*;
                match key {
                    Q | Left => self.going_left = state == Pressed,
                    D | Right => self.going_right = state == Pressed,
                    _ => {}
                }
            },
            MouseMoved(x, y) => {
                let (w, h, _, _) = self.bundle.data.color_target.get_dimensions();
                self.mouse = Vector2::new((x as f32 / w as f32) - 0.5,
                                          0.5 - (y as f32 / h as f32));
            }
            MouseWheel(_delta, _) => {
                
            }
            MouseInput(state, Left) => self.head_spinning = state == Pressed,
            _ => {}
        }
    }
}

fn main() {
    let wb = winit::WindowBuilder::new().with_title("Primus Polygoni");
    gfx_app::launch_gl3::<gfx_app::Wrap<_,_,App<_>>>(wb);
}
