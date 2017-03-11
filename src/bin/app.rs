extern crate primus_polygoni;
extern crate image;
extern crate time;

use std::f32::consts::PI;
use primus_polygoni::gfx;
use primus_polygoni::gfx_app;
use primus_polygoni::winit;
use primus_polygoni::nalgebra::{Vector2, Vector3, UnitQuaternion};
use time::precise_time_s;

use primus_polygoni::{Vertex, Locals, Camera};

const SCENE_SPHERES: f32 = 1.0;
const SCENE_RADIUS: f32 = 3.0;

struct App<R: gfx::Resources> {
    bundle: gfx::Bundle<R, primus_polygoni::pipe::Data<R>>,
    camera: Camera,
    aspect_ratio: f32,
    mouse: Vector2<f32>,
    head_spinning: bool,
    going_left: bool,
    going_right: bool,
    marker: f32,
}

impl<R: gfx::Resources> gfx_app::Application<R> for App<R> {
    fn new<F: gfx::Factory<R>>(factory: &mut F,
                               backend: gfx_app::shade::Backend,
                               targets: gfx_app::WindowTargets<R>) -> Self {
        use gfx::traits::FactoryExt;

        let (vertex_data, index_data) = primus_polygoni::generate_icosphere(4);

        let (vertices, slice) = factory
            .create_vertex_buffer_with_slice(&vertex_data[..], &index_data[..]);
        let pso = primus_polygoni::create_pipeline(factory, backend);

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
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
        let now = precise_time_s() as f32;
        let delta = now - self.marker;
        self.marker = now;

        if self.head_spinning {
            let max_rotation = 2.0 * PI * delta;
            self.camera.rotate(UnitQuaternion::new(
                Vector3::y() * (-self.mouse.x * max_rotation)
            ));
            self.camera.pitch(self.mouse.y * max_rotation);
        }
        
        let speed = (2.0 * PI) / SCENE_SPHERES;
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
    use gfx_app::Application;
    App::launch_simple("Primus Polygoni");
}
