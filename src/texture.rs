use std::f32;
use rand::{self, Rng};
use noise::{
    NoiseModule, Seedable, MultiFractal, Constant, Fbm, Billow, RidgedMulti
};

pub fn generate(output: &mut [[u8; 4]], size: usize) {
    let mut rng = rand::thread_rng();

    let mut choices = [0, 0, 1, 1, 2, 2, 3, 3];
    rng.shuffle(&mut choices);

    let mut gen_noise = |i: usize| {
        match choices[i] {
            0 => Box::new(Constant::new(rng.gen_range(0.05f32, 0.95)))
                    as Box<NoiseModule<[f32; 3], Output=f32> + Sync>,
            1 => Box::new(Fbm::new()
                    .set_seed(rng.gen())
                    .set_octaves(rng.gen_range(0, 4))
                    .set_frequency(rng.gen_range(0.5, 2.0))
                    .set_lacunarity(rng.gen_range(1.5, 2.5))
                    .set_persistence(rng.gen_range(0.2, 1.0))),
            2 => Box::new(Billow::new()
                    .set_seed(rng.gen())
                    .set_octaves(rng.gen_range(0, 4))
                    .set_frequency(rng.gen_range(0.5, 2.0))
                    .set_lacunarity(rng.gen_range(1.5, 2.5))
                    .set_persistence(rng.gen_range(0.2, 1.0))),
            _ => Box::new(RidgedMulti::new()
                    .set_seed(rng.gen())
                    .set_octaves(rng.gen_range(0, 4))
                    .set_frequency(rng.gen_range(0.5, 2.0))
                    .set_lacunarity(rng.gen_range(1.5, 2.5))
                    .set_persistence(rng.gen_range(0.5, 1.0))
                    .set_attenuation(rng.gen_range(1.7, 2.3))),
        }
    };

    fill(&*gen_noise(0), &*gen_noise(1), &*gen_noise(2), output, size);
}

fn fill<R, G, B>(r: &R, g: &G, b: &B, output: &mut [[u8; 4]], size: usize)
    where R: NoiseModule<[f32; 3], Output=f32> + ?Sized + Sync,
          G: NoiseModule<[f32; 3], Output=f32> + ?Sized + Sync,
          B: NoiseModule<[f32; 3], Output=f32> + ?Sized + Sync
{
    use rayon::prelude::*;

    debug_assert!(output.len() == 2 * size * size);
    output.par_chunks_mut(2 * size).enumerate().for_each(|(y, line)| {
        for (x, texel) in line.iter_mut().enumerate() {
            let theta = (x as f32 / size as f32) * f32::consts::PI;
            let phi = -(y as f32 / size as f32) * f32::consts::PI;
            let p = [phi.sin() * theta.cos(),
                     phi.sin() * theta.sin(),
                     phi.cos()];
            *texel = [noise_to_u8(r.get(p)),
                      noise_to_u8(g.get(p)),
                      noise_to_u8(b.get(p)),
                      0xFF];
        }
    });
}

fn noise_to_u8(v: f32) -> u8 {
    ((v + 1.0) * 255. * 0.5) as u8
}