use std::f32;
use rand::{self, Rng};
use noise::{
    NoiseModule, Seedable, MultiFractal, Constant, Perlin, Billow, RidgedMulti
};

pub fn generate(output: &mut [[u8; 4]], size: usize) {
    let mut rng = rand::thread_rng();

    let mut gen_noise = || {
        match rng.gen_range(0, 4) {
            0 => Box::new(Constant::new(rng.gen_range(0f32, 1f32))) as Box<NoiseModule<[f32; 3], Output=f32>>,
            1 => Box::new(Perlin::new().set_seed(rng.gen())),
            2 => Box::new(Billow::new()
                    .set_seed(rng.gen())
                    .set_octaves(rng.gen_range(2, 6))
                    .set_frequency(rng.gen_range(0.8, 1.2))
                    .set_persistence(rng.gen_range(0.3, 0.8))),
            _ => Box::new(RidgedMulti::new()
                    .set_seed(rng.gen())
                    .set_octaves(rng.gen_range(2, 6))
                    .set_frequency(rng.gen_range(0.8, 1.2))
                    .set_persistence(rng.gen_range(0.7, 1.3))
                    .set_attenuation(rng.gen_range(1.7, 2.3))),
        } 
    }; 

    fill(&*gen_noise(), &*gen_noise(), &*gen_noise(), output, size);
}

fn fill<R, G, B>(r: &R, g: &G, b: &B, output: &mut [[u8; 4]], size: usize)
    where R: NoiseModule<[f32; 3], Output=f32> + ?Sized,
          G: NoiseModule<[f32; 3], Output=f32> + ?Sized,
          B: NoiseModule<[f32; 3], Output=f32> + ?Sized
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