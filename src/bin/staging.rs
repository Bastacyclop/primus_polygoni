extern crate primus_polygoni;
use primus_polygoni::{gfx, generate_texture, Scene};
use primus_polygoni::gfx::traits::FactoryExt;
use primus_polygoni::gfx::memory::Typed;

struct Impl<R: gfx::Resources> {
    upload: gfx::handle::Buffer<R, [u8; 4]>
}

impl<R: gfx::Resources> primus_polygoni::scene::Impl<R> for Impl<R> {
    fn init<F: gfx::Factory<R>>(w: usize,
                                h: usize,
                                a: usize,
                                factory: &mut F) -> Self {
        Impl {
            upload: factory.create_upload_buffer(w * h * a)
                .expect("could not create upload buffer")
        }
    }

    fn texture_bind() -> gfx::memory::Bind {
        gfx::memory::TRANSFER_DST
    }

    fn texture_usage() -> gfx::memory::Usage {
        gfx::memory::Usage::Data
    }

    fn generate_textures<C, F>(scene: &mut Scene<R, Self>,
                               encoder: &mut gfx::Encoder<R, C>,
                               factory: &mut F)
        where C: gfx::CommandBuffer<R>, F: gfx::Factory<R>
    {
        let (w, h) = (scene.texture_size * 2, scene.texture_size);
        let mut writer = factory.write_mapping(&scene.implementation.upload)
            .expect("could not write to mapping");

        for texels in writer.chunks_mut(w * h) {
            generate_texture(texels, scene.texture_size);
        }

        let info = gfx::texture::ImageInfoCommon {
            xoffset: 0,
            yoffset: 0,
            zoffset: 0,
            width: w as gfx::texture::Size,
            height: h as gfx::texture::Size,
            depth: scene.sphere_count as gfx::texture::Size,
            format: <gfx::format::Rgba8 as gfx::format::Formatted>::get_format(),
            mipmap: 0
        };

        encoder.copy_buffer_to_texture_raw(
            scene.implementation.upload.raw(), 0,
            scene.texture.raw(), None, info
        ).unwrap();
    }
}

fn main() {
    primus_polygoni::run::<Impl<_>>("memory staging (Upload + Data)");
}
