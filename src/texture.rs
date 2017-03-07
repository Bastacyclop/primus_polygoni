use std::f32;
use noise::{
    NoiseModule, Constant, Checkerboard,
    Fbm, HybridMulti, RidgedMulti, BasicMulti, Perlin, Worley, Billow
};

pub fn generate(output: &mut [[u8; 4]], size: usize) {
    fill(&Billow::new(), &Constant::new(-0.2), &Constant::new(-0.3), output, size);
}

fn fill<R, G, B>(r: &R, g: &G, b: &B, output: &mut [[u8; 4]], size: usize)
    where R: NoiseModule<[f32; 3], Output=f32>,
          G: NoiseModule<[f32; 3], Output=f32>,
          B: NoiseModule<[f32; 3], Output=f32>
{
    debug_assert!(output.len() == 2 * size * size);
    for y in 0..size {
        for x in 0..(2 * size) {
            let theta = (x as f32 / size as f32) * f32::consts::PI;
            let phi = -(y as f32 / size as f32) * f32::consts::PI;
            let p = [phi.sin() * theta.cos(),
                     phi.sin() * theta.sin(),
                     phi.cos()];
            output[y*2*size + x] = [noise_to_u8(r.get(p)),
                                    noise_to_u8(g.get(p)),
                                    noise_to_u8(b.get(p)),
                                    0xFF];
        }
    }
}

fn noise_to_u8(v: f32) -> u8 {
    ((v + 1.0) * 255. * 0.5) as u8
}