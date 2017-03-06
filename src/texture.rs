use noise::{
    NoiseModule, Constant,
    Fbm, HybridMulti, RidgedMulti, BasicMulti, Perlin, Worley, Billow
};

pub fn generate(output: &mut [[u8; 4]], size: usize) {
    fill(&Billow::new(), &Constant::new(-0.2), &RidgedMulti::new(), output, size);
}

fn fill<R, G, B>(r: &R, g: &G, b: &B, output: &mut [[u8; 4]], size: usize)
    where R: NoiseModule<[f32; 2], Output=f32>,
          G: NoiseModule<[f32; 2], Output=f32>,
          B: NoiseModule<[f32; 2], Output=f32>
{
    debug_assert!(output.len() == size * size);
    let noise_size = 10.;
    for y in 0..size {
        for x in 0..size {
            let p = [noise_size * x as f32 / size as f32,
                     noise_size * y as f32 / size as f32];
            output[y*size + x] = [noise_to_u8(r.get(p)),
                                  noise_to_u8(g.get(p)),
                                  noise_to_u8(b.get(p)),
                                  0xFF];
        }
    }
}

fn noise_to_u8(v: f32) -> u8 {
    ((v + 1.0) * 255. * 0.5) as u8
}