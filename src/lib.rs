#[macro_use]
pub extern crate gfx;
pub extern crate gfx_app;
pub extern crate winit;
pub extern crate alga;
pub extern crate nalgebra;
extern crate rand;
extern crate noise;

mod camera;
mod texture;
mod icosphere;

pub use camera::Camera;
pub use texture::generate as generate_texture;
pub use icosphere::generate as generate_icosphere;
pub use gfx_app::{ColorFormat, DepthFormat};

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vertices: gfx::VertexBuffer<Vertex> = (),
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        color_target: gfx::RenderTarget<ColorFormat> = "Target0",
        depth_target: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    pub fn new(pos: [f32; 3], tex_coord: [f32; 2]) -> Vertex {
        Vertex {
            pos: [pos[0], pos[1], pos[2], 1.0],
            tex_coord: tex_coord,
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
        hlsl_40: include_bytes!("../data/vertex.fx"),
        .. gfx_app::shade::Source::empty()
    };

    let fs = gfx_app::shade::Source {
        glsl_150: include_bytes!("shader/main_150.glslf"),
        hlsl_40: include_bytes!("../data/pixel.fx"),
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
