use std::collections::HashMap;
use std::mem;
use scene::Vertex;

fn vertex(pos: [f32; 3]) -> Vertex {
    use std::f32::consts::{PI};

    let u = pos[0].atan2(pos[2]) / (-2.0 * PI);
    let u = if u < 0. { u + 1. } else { u };
    let v = pos[1].asin() / PI + 0.5;

    Vertex::new(pos, [u, v])
}

pub fn generate(recursion: u16) -> (Vec<Vertex>, Vec<u16>) {
    let face_count = 20 * 4usize.pow(recursion as u32);
    let edge_count = 3 * face_count / 2;
    // Euler's formula
    let vertex_count = 2 + edge_count - face_count;
    let index_count = face_count * 3;

    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let n = (1. +  t * t).sqrt();
    let u = 1. / n;
    let v = t / n;

    let mut vertex_data = Vec::with_capacity(vertex_count);
    vertex_data.extend_from_slice(&[
        vertex([-u, v, 0.0]),
        vertex([ u, v, 0.0]),
        vertex([ -u, -v, 0.0]),
        vertex([ u, -v, 0.0]),

        vertex([ 0.0, -u, v]),
        vertex([ 0.0, u, v]),
        vertex([ 0.0, -u, -v]),
        vertex([ 0.0, u, -v]),

        vertex([ v, 0.0, -u]),
        vertex([ v, 0.0, u]),
        vertex([ -v, 0.0, -u]),
        vertex([ -v, 0.0, u]),
    ]);

    let mut index_data: Vec<u16> = Vec::with_capacity(index_count);
    index_data.extend_from_slice(&[
        // 5 faces around point 0
        0, 11, 5,
        0, 5, 1,
        0, 1, 7,
        0, 7, 10,
        0, 10, 11,

        // 5 adjacent faces
        1, 5, 9,
        5, 11, 4,
        11, 10, 2,
        10, 7, 6,
        7, 1, 8,

        // 5 faces around point 3
        3, 9, 4,
        3, 4, 2, 
        3, 2, 6,
        3, 6, 8, 
        3, 8, 9, 

        // 5 adjacent faces
        4, 9, 5,
        2, 4, 11,
        6, 2, 10,
        8, 6, 7,
        9, 8, 1,
    ]);

    let mut cache = HashMap::new();
    let mut next_indices = Vec::with_capacity(index_count);

    {
        let mut middle = |ia, ib| {
            let key = if ia < ib { (ia, ib) } else { (ib, ia) };

            cache.get(&key).cloned().unwrap_or_else(|| {
                let pa = vertex_data[ia as usize].pos;
                let pb = vertex_data[ib as usize].pos;
                let middle = [
                    (pa[0] + pb[0]) / 2.0, 
                    (pa[1] + pb[1]) / 2.0,
                    (pa[2] + pb[2]) / 2.0,
                ];
                let norm = (middle[0] * middle[0] +
                            middle[1] * middle[1] +
                            middle[2] * middle[2]).sqrt();
                
                let index = vertex_data.len() as u16;
                let v = vertex([middle[0]/norm, middle[1]/norm, middle[2]/norm]);
                vertex_data.push(v);

                cache.insert(key, index);
                index
            }) 
        };

        for _ in 0..recursion {
            for tri in index_data.chunks(3) {
                let i1 = tri[0];
                let i2 = tri[1];
                let i3 = tri[2];

                let a = middle(i1, i2);
                let b = middle(i2, i3);
                let c = middle(i3, i1);

                next_indices.extend_from_slice(&[
                    i1, a, c,
                    a, i2, b,
                    c, b, i3,
                    a, b, c,   
                ]);
            }
            mem::swap(&mut next_indices, &mut index_data);
            next_indices.clear();
        }
    }

    debug_assert!(vertex_data.len() == vertex_count);
    debug_assert!(index_data.len() == index_count);
    (vertex_data, index_data)
}
