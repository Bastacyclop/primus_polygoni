extern crate primus_polygoni;
use primus_polygoni::gfx;
use primus_polygoni::gfx_app;
use primus_polygoni::winit;

use primus_polygoni::{Vertex, Locals, Camera};

struct App<R: gfx::Resources> {
    bundle: gfx::Bundle<R, primus_polygoni::pipe::Data<R>>,
    camera: Camera,
    aspect_ratio: f32,
}

impl<R: gfx::Resources> gfx_app::Application<R> for App<R> {
    fn new<F: gfx::Factory<R>>(factory: &mut F,
                               backend: gfx_app::shade::Backend,
                               targets: gfx_app::WindowTargets<R>) -> Self {
        use gfx::traits::FactoryExt;

        let vertex_data = &[
            // top (0, 0, 0.5)
            Vertex::new([-0.5, -0.5,  0.5], [0.9, 0.2, 0.1]),
            Vertex::new([ 0.5, -0.5,  0.5], [0.9, 0.2, 0.1]),
            Vertex::new([ 0.5,  0.5,  0.5], [0.9, 0.2, 0.1]),
            Vertex::new([-0.5,  0.5,  0.5], [0.9, 0.2, 0.1]),
            // bottom (0, 0, -0.5)
            Vertex::new([-0.5,  0.5, -0.5], [0.1, 0.8, 0.9]),
            Vertex::new([ 0.5,  0.5, -0.5], [0.1, 0.8, 0.9]),
            Vertex::new([ 0.5, -0.5, -0.5], [0.1, 0.8, 0.9]),
            Vertex::new([-0.5, -0.5, -0.5], [0.1, 0.8, 0.9]),
            // right (0.5, 0, 0)
            Vertex::new([ 0.5, -0.5, -0.5], [0.2, 0.9, 0.1]),
            Vertex::new([ 0.5,  0.5, -0.5], [0.2, 0.9, 0.1]),
            Vertex::new([ 0.5,  0.5,  0.5], [0.2, 0.9, 0.1]),
            Vertex::new([ 0.5, -0.5,  0.5], [0.2, 0.9, 0.1]),
            // left (-0.5, 0, 0)
            Vertex::new([-0.5, -0.5,  0.5], [0.8, 0.1, 0.9]),
            Vertex::new([-0.5,  0.5,  0.5], [0.8, 0.1, 0.9]),
            Vertex::new([-0.5,  0.5, -0.5], [0.8, 0.1, 0.9]),
            Vertex::new([-0.5, -0.5, -0.5], [0.8, 0.1, 0.9]),
            // front (0, 0.5, 0)
            Vertex::new([ 0.5,  0.5, -0.5], [0.2, 0.1, 0.9]),
            Vertex::new([-0.5,  0.5, -0.5], [0.2, 0.1, 0.9]),
            Vertex::new([-0.5,  0.5,  0.5], [0.2, 0.1, 0.9]),
            Vertex::new([ 0.5,  0.5,  0.5], [0.2, 0.1, 0.9]),
            // back (0, -0.5, 0)
            Vertex::new([ 0.5, -0.5,  0.5], [0.8, 0.9, 0.1]),
            Vertex::new([-0.5, -0.5,  0.5], [0.8, 0.9, 0.1]),
            Vertex::new([-0.5, -0.5, -0.5], [0.8, 0.9, 0.1]),
            Vertex::new([ 0.5, -0.5, -0.5], [0.8, 0.9, 0.1]),
        ];

        let index_data: &[u16] = &[
            0,  1,  2,  2,  3,  0, // top
            4,  5,  6,  6,  7,  4, // bottom
            8,  9, 10, 10, 11,  8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        let (vertices, slice) = factory
            .create_vertex_buffer_with_slice(vertex_data, index_data);
        let pso = primus_polygoni::create_pipeline(factory, backend);

        let data = primus_polygoni::pipe::Data {
            vertices: vertices,
            locals: factory.create_constant_buffer(1),
            color_target: targets.color,
            depth_target: targets.depth,
        };

        App {
            bundle: gfx::Bundle::new(slice, pso, data),
            camera: Camera::new(),
            aspect_ratio: targets.aspect_ratio,
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
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
                // let (w, h) = window.get_inner_size_pixels();
                let (w, h, _, _) = self.bundle.data.color_target.get_dimensions();
                let dx = (x as f32 / w as f32) - 0.5;
                let dy = (y as f32 / h as f32) - 0.5;
                self.camera.yaw(-dx / 60.0);
                self.camera.pitch(-dy / 60.0);
                // window.set_cursor_position(w / 2, h / 2)
            }
            MouseWheel(_delta, _) => {
                
            }
            MouseInput(_state, _button) => {
                
            }
            _ => {}
        }
    }
}

fn main() {
    use gfx_app::Application;
    App::launch_simple("Primus Polygoni");
}
