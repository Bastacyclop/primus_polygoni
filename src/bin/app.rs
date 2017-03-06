extern crate primus_polygoni;
use primus_polygoni::gfx;
use primus_polygoni::gfx_app;
use primus_polygoni::winit;
use primus_polygoni::nalgebra::{Vector2, Vector3, UnitQuaternion};

use primus_polygoni::{Vertex, Locals, Camera};

struct App<R: gfx::Resources> {
    bundle: gfx::Bundle<R, primus_polygoni::pipe::Data<R>>,
    camera: Camera,
    aspect_ratio: f32,
    mouse: Vector2<f32>,
    head_spinning: bool,
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

        let size = 256;
        let mut texels: Vec<_> = (0..(size * size)).map(|_| [0xFF, 0xFF, 0xFF, 0xFF])
            .collect();
        primus_polygoni::generate_texture(&mut texels, size);

        let (_, texture_view) =
            factory.create_texture_immutable::<gfx::format::Rgba8>(
                gfx::texture::Kind::D2(size as gfx::texture::Size, 
                                       size as gfx::texture::Size, 
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
            camera: Camera::new(),
            aspect_ratio: targets.aspect_ratio,
            mouse: Vector2::new(0., 0.),
            head_spinning: false,
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
        if self.head_spinning {
            self.camera.rotate(UnitQuaternion::new(
                Vector3::y() * (-self.mouse.x / 10.0)
            ));
            self.camera.pitch(-self.mouse.y / 10.0);
        }

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
            KeyboardInput(Pressed, _, Some(key)) => {
                use winit::VirtualKeyCode::*;
                let move_step = 0.05;
                let angle_step = 0.314;
                match key {
                    Left => self.camera.move_right(-move_step),
                    Right => self.camera.move_right(move_step),
                    Down => self.camera.move_ahead(-move_step),
                    Up => self.camera.move_ahead(move_step),
                    PageDown => self.camera.move_up(-move_step),
                    PageUp => self.camera.move_up(move_step),
                    Z => self.camera.pitch(angle_step),
                    S => self.camera.pitch(-angle_step),
                    Q => self.camera.yaw(angle_step),
                    D => self.camera.yaw(-angle_step),
                    A => self.camera.roll(angle_step),
                    E => self.camera.roll(-angle_step),
                    _ => {}
                }
            },
            MouseMoved(x, y) => {
                let (w, h, _, _) = self.bundle.data.color_target.get_dimensions();
                self.mouse = Vector2::new((x as f32 / w as f32) - 0.5,
                                          (y as f32 / h as f32) - 0.5);
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
