extern crate primus_polygoni;
use primus_polygoni::{gfx, generate_texture, Scene};

struct Impl;
impl<R: gfx::Resources> primus_polygoni::scene::Impl<R> for Impl {
    fn init<F: gfx::Factory<R>>(_w: usize,
                                _h: usize,
                                _a: usize,
                                _factory: &mut F) -> Self {
        Impl
    }

    fn texture_bind() -> gfx::memory::Bind {
        gfx::memory::Bind::empty()
    }

    fn texture_usage() -> gfx::memory::Usage {
        gfx::memory::Usage::Dynamic
    }

    fn generate_textures<C, F>(scene: &mut Scene<R, Self>,
                               encoder: &mut gfx::Encoder<R, C>,
                               _: &mut F)
        where C: gfx::CommandBuffer<R>, F: gfx::Factory<R>
    {
        let (w, h) = (scene.texture_size * 2, scene.texture_size);
        let mut texels: Vec<_> = (0..(w * h)).map(|_| [0; 4]).collect();
        let mut info = gfx::texture::ImageInfoCommon {
            xoffset: 0,
            yoffset: 0,
            zoffset: 0,
            width: w as gfx::texture::Size,
            height: h as gfx::texture::Size,
            depth: 1,
            format: (),
            mipmap: 0
        };

        for i in 0..scene.sphere_count {
            generate_texture(&mut texels[..], scene.texture_size);
            info.zoffset = i as gfx::texture::Size;
            encoder.update_texture::<_, gfx::format::Rgba8>
                (&scene.texture, None, info, &texels[..])
                .unwrap();
        }
    }
}

fn main() {
    primus_polygoni::run::<Impl>("simple memory update (Dynamic)");
}
