use std::f32::consts::PI;
use {gfx, rand};
use rand::distributions::{IndependentSample, Range};
use nalgebra::{self, Vector3};
use Camera;

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

pub trait Impl<R: gfx::Resources>: Sized {
    fn init<F: gfx::Factory<R>>(_w: usize,
                                _h: usize,
                                _a: usize,
                                _factory: &mut F) -> Self;
    fn texture_bind() -> gfx::memory::Bind;
    fn texture_usage() -> gfx::memory::Usage;
    fn generate_textures<C, F>(scene: &mut Scene<R, Self>,
                               encoder: &mut gfx::Encoder<R, C>,
                               factory: &mut F)
        where C: gfx::CommandBuffer<R>, F: gfx::Factory<R>;
}

pub struct Scene<R: gfx::Resources, I: Impl<R>> {
    pub sphere_count: usize,
    pub scene_radius: f32,
    pub texture_size: usize,
    pub camera: Camera,
    pub pso: gfx::PipelineState<R, pipe::Meta>,
    pub data: pipe::Data<R>,
    pub slice: gfx::Slice<R>,
    pub texture: gfx::handle::Texture<R, gfx::format::R8_G8_B8_A8>,
    pub implementation: I,
}

impl<R: gfx::Resources, I: Impl<R>> Scene<R, I> {
    pub fn new<F, C>(sphere_count: usize,
                     texture_size: usize,
                     color_target: gfx::handle::RenderTargetView<R, ColorFormat>,
                     depth_target: gfx::handle::DepthStencilView<R, DepthFormat>,
                     factory: &mut F,
                     encoder: &mut gfx::Encoder<R, C>) -> Self
        where F: gfx::Factory<R>, C: gfx::CommandBuffer<R>
    {
        use gfx::traits::FactoryExt;

        let program = factory.link_program(
            include_bytes!("shader/main_150.glslv"),
            include_bytes!("shader/main_150.glslf")
        ).expect("could not create scene program");

        let pso = factory.create_pipeline_from_program(
            &program,
            gfx::Primitive::TriangleList,
            gfx::state::Rasterizer::new_fill(),
            pipe::new()
        ).expect("could not create scene pipeline");

        let (vertex_data, index_data) = ::generate_icosphere(4);
        let (vertices, mut slice) = factory
            .create_vertex_buffer_with_slice(&vertex_data[..], &index_data[..]);
        slice.instances = Some((sphere_count as u32, 0));

        let instances = factory.create_buffer(sphere_count,
                                              gfx::buffer::Role::Vertex,
                                              gfx::memory::Usage::Dynamic,
                                              gfx::Bind::empty()).unwrap();

        let (w, h) = (texture_size * 2, texture_size);
        let texture =
            factory.create_texture(
                gfx::texture::Kind::D2Array(w as gfx::texture::Size,
                                            h as gfx::texture::Size,
                                            sphere_count as gfx::texture::Size,
                                            gfx::texture::AaMode::Single),
                1,
                gfx::memory::SHADER_RESOURCE | I::texture_bind(),
                I::texture_usage(),
                Some(gfx::format::ChannelType::Unorm)
            ).expect("could not create scene texture");
        let texture_view = factory
            .view_texture_as_shader_resource::<gfx::format::Rgba8>(&texture, (0, 0), gfx::format::Swizzle::new())
            .expect("could not create scene texture view");

        let sinfo = gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Bilinear,
            gfx::texture::WrapMode::Clamp);

        let radius = (sphere_count as f32 * 4.0) / (2.0 * PI);
        let scene = Scene {
            sphere_count: sphere_count,
            scene_radius: radius,
            texture_size: texture_size,
            camera: Camera::new(radius),
            pso: pso,
            data: pipe::Data {
                vertices: vertices,
                instances: instances,
                locals: factory.create_constant_buffer(1),
                color: (texture_view, factory.create_sampler(sinfo)),
                color_target: color_target,
                depth_target: depth_target,
            },
            slice: slice,
            texture: texture,
            implementation: I::init(w, h, sphere_count, factory),
        };

        scene.fill_instances(encoder);
        scene
    }

    fn fill_instances<C>(&self, encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        let mut vec = Vec::with_capacity(self.sphere_count);
        let mut rng = rand::thread_rng();

        for i in 0..self.sphere_count {
            let angle = i as f32 / self.sphere_count as f32 * (2.0 * PI);
            let position = Vector3::new(
                angle.cos() * self.scene_radius,
                0.0,
                angle.sin() * self.scene_radius);

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
        encoder.update_buffer(&self.data.instances, &vec[..], 0).unwrap();
    }

    pub fn generate_textures<C, F>(&mut self,
                                   encoder: &mut gfx::Encoder<R, C>,
                                   factory: &mut F)
        where C: gfx::CommandBuffer<R>, F: gfx::Factory<R>
    {
        println!("Generating textures");
        I::generate_textures(self, encoder, factory);
    }

    pub fn render<C>(&mut self,
                     aspect_ratio: f32,
                     encoder: &mut gfx::Encoder<R, C>)
        where C: gfx::CommandBuffer<R>
    {
        self.camera.update(aspect_ratio);
        encoder.update_constant_buffer(&self.data.locals, &Locals {
            transform: self.camera.gpu_transform()
        });

        encoder.draw(&self.slice, &self.pso, &self.data);
    }
}
