#[macro_use]
pub extern crate gfx;
pub extern crate winit;
pub extern crate alga;
pub extern crate nalgebra;
pub extern crate rand;
extern crate noise;

mod camera;
mod texture;
mod icosphere;

pub use camera::Camera;
pub use texture::generate as generate_texture;
pub use icosphere::generate as generate_icosphere;
pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    vertex Instance {
        t1: [f32; 4] = "a_T1",
        t2: [f32; 4] = "a_T2",
        t3: [f32; 4] = "a_T3",
        t4: [f32; 4] = "a_T4",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vertices: gfx::VertexBuffer<Vertex> = (),
        instances: gfx::InstanceBuffer<Instance> = (),
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

pub fn create_pipeline<R, F>(factory: &mut F)
                             -> gfx::PipelineState<R, pipe::Meta>
    where R: gfx::Resources, F: gfx::Factory<R>
{
    use gfx::traits::FactoryExt;

    let program = factory.link_program(
        include_bytes!("shader/main_150.glslv"),
        include_bytes!("shader/main_150.glslf")).unwrap();

    factory.create_pipeline_from_program(
        &program,
        gfx::Primitive::TriangleList,
        gfx::state::Rasterizer::new_fill(),
        pipe::new()
    ).unwrap()
}
