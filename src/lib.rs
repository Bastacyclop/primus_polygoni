#[macro_use]
pub extern crate gfx;
pub extern crate gfx_app;
pub extern crate winit;
pub extern crate alga;
pub extern crate nalgebra;
extern crate rand;

mod camera;

pub use camera::Camera;
pub use gfx_app::{ColorFormat, DepthFormat};

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vertices: gfx::VertexBuffer<Vertex> = (),
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        color_target: gfx::RenderTarget<ColorFormat> = "Target0",
        depth_target: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    pub fn new(pos: [f32; 3], color: [f32; 3]) -> Vertex {
        Vertex {
            pos: [pos[0], pos[1], pos[2], 1.0],
            color: color,
        }
    }
}

pub fn create_pipeline<R, F>(factory: &mut F,
                             backend: gfx_app::shade::Backend)
                             -> gfx::PipelineState<R, pipe::Meta>
    where R: gfx::Resources, F: gfx::Factory<R>
{
    use gfx::traits::FactoryExt;

    let vs = gfx_app::shade::Source {
        glsl_150: include_bytes!("shader/main_150.glslv"),
        .. gfx_app::shade::Source::empty()
    };

    let fs = gfx_app::shade::Source {
        glsl_150: include_bytes!("shader/main_150.glslf"),
        .. gfx_app::shade::Source::empty()
    };

    let program = factory.link_program(vs.select(backend).unwrap(),
                                       fs.select(backend).unwrap()).unwrap();

    factory.create_pipeline_from_program(
        &program,
        gfx::Primitive::TriangleList,
        gfx::state::Rasterizer::new_fill(),
        pipe::new()
    ).unwrap()
}
