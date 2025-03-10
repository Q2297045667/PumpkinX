use pumpkin_data::chunk::Biome;
use pumpkin_util::math::{floor_mod, square, vector3::Vector3};

use crate::ProtoChunk;

pub fn get_biome(chunk: &ProtoChunk, seed: i64, pos: &Vector3<i32>) -> Biome {
    let i = pos.x - 2;
    let j = pos.y - 2;
    let k = pos.z - 2;
    let l = i >> 2;
    let m = j >> 2;
    let n = k >> 2;
    let d = (i & 3) as f64 / 4.0;
    let e = (j & 3) as f64 / 4.0;
    let f = (k & 3) as f64 / 4.0;
    let mut o = 0;
    let mut g = f64::INFINITY;
    for p in 0..8 {
        let bl = (p & 4) == 0;
        let q = if bl { l } else { l + 1 };
        let bl2 = (p & 2) == 0;
        let r = if bl2 { m } else { m + 1 };
        let bl3 = (p & 1) == 0;
        let s = if bl3 { n } else { n + 1 };
        let h = if bl { d } else { d - 1.0 };
        let t = if bl2 { e } else { e - 1.0 };
        let u = if bl3 { f } else { f - 1.0 };
        let v = method_38106(seed, q, r, s, h, t, u);
        if g > v {
            o = p;
            g = v;
        }
    }
    let x = if (o & 4) == 0 { l } else { l + 1 };
    let y = if (o & 2) == 0 { m } else { m + 1 };
    let z = if (o & 1) == 0 { n } else { n + 1 };
    chunk.get_biome(&Vector3::new(x, y, z))
}

fn method_38106(seed: i64, i: i32, salt: i32, k: i32, d: f64, e: f64, f: f64) -> f64 {
    let mut m = seed;
    m = mix_seed(m, i as i64);
    m = mix_seed(m, salt as i64);
    m = mix_seed(m, k as i64);
    m = mix_seed(m, i as i64);
    m = mix_seed(m, salt as i64);
    m = mix_seed(m, k as i64);
    let g = method_38108(m);
    m = mix_seed(m, seed);
    let h = method_38108(m);
    m = mix_seed(m, seed);
    let n = method_38108(m);
    square(f + n) +square(e + h) +square(d + g)
}

fn method_38108(l: i64) -> f64 {
    let d = floor_mod(l >> 24, 1024) as f64 / 1024.0;
    (d - 0.5) * 0.9
}

fn mix_seed(mut seed: i64, salt: i64) -> i64 {
    seed *= seed * 6364136223846793005 + 1442695040888963407;
    return seed + salt;
}
