use pumpkin_data::chunk::Biome;
use pumpkin_util::math::{floor_mod, square, vector3::Vector3};

use crate::ProtoChunk;

use super::{biome_coords, height_limit::HeightLimitView};

// This blends biome boundaries, returning which biome to populate the surface on based on the seed
pub fn get_biome_blend(chunk: &ProtoChunk, seed: u64, pos: &Vector3<i32>) -> Biome {
    // This is the "left" side of the biome boundary
    let offset_x = pos.x - 2;
    let offset_y = pos.y - 2;
    let offset_z = pos.z - 2;
    let biome_x = biome_coords::from_block(offset_x);
    let biome_y = biome_coords::from_block(offset_y);
    let biome_z = biome_coords::from_block(offset_z);
    // &'ing 3 gives values of 0-3, it is also the data we removed when converting to biome coords
    // This is effectively "quarters" into the biome
    let biome_x_quarters = (offset_x & 0b11) as f64 / 4.0;
    let biome_y_quarters = (offset_y & 0b11) as f64 / 4.0;
    let biome_z_quarters = (offset_z & 0b11) as f64 / 4.0;

    let mut best_permutation = 0;
    let mut best_score = f64::INFINITY;
    for permutation in 0..8 {
        let should_maintain_x = (permutation & 0b100) == 0;
        let should_maintain_y = (permutation & 0b010) == 0;
        let should_maintain_z = (permutation & 0b001) == 0;

        // If we are shifting, add 1 to the biome coords
        let shifted_biome_x = if should_maintain_x {
            biome_x
        } else {
            biome_x + 1
        };
        let shifted_biome_y = if should_maintain_y {
            biome_y
        } else {
            biome_y + 1
        };
        let shifted_biome_z = if should_maintain_z {
            biome_z
        } else {
            biome_z + 1
        };

        // And reflect the "quarters" across the shift
        let shifted_biome_x_quarters = if should_maintain_x {
            biome_x_quarters
        } else {
            biome_x_quarters - 1.0
        };
        let shifted_biome_y_quarters = if should_maintain_y {
            biome_y_quarters
        } else {
            biome_y_quarters - 1.0
        };
        let shifted_biome_z_quarters = if should_maintain_z {
            biome_z_quarters
        } else {
            biome_z_quarters - 1.0
        };

        let permutation_score = score_permutation(
            seed as i64,
            shifted_biome_x,
            shifted_biome_y,
            shifted_biome_z,
            shifted_biome_x_quarters,
            shifted_biome_y_quarters,
            shifted_biome_z_quarters,
        );

        if best_score > permutation_score {
            best_permutation = permutation;
            best_score = permutation_score;
        }
    }

    // Now check if we want to use the "left" side or the "right" side
    let biome_x = if (best_permutation & 0b100) == 0 {
        biome_x
    } else {
        biome_x + 1
    };
    let biome_y = if (best_permutation & 0b010) == 0 {
        biome_y
    } else {
        biome_y + 1
    };
    let biome_z = if (best_permutation & 0b001) == 0 {
        biome_z
    } else {
        biome_z + 1
    };

    // Java's `getBiomeForNoiseGen`
    let bottom_y = chunk.bottom_y() as i32;
    let biome_bottom = biome_coords::from_block(bottom_y);
    let biome_top = biome_bottom + biome_coords::from_block(chunk.height() as i32) - 1;
    let biome_y = biome_y.clamp(biome_bottom, biome_top);

    chunk.get_biome(&Vector3::new(biome_x, biome_y, biome_z))
}

// This is effectively getting a random offset (+/- 0.0-0.8ish) to our biome position quarters and
// returning a hypotenuse squared of the parts + the offset
fn score_permutation(
    seed: i64,
    x: i32,
    y: i32,
    z: i32,
    x_part: f64,
    y_part: f64,
    z_part: f64,
) -> f64 {
    let mix = salt_mix(seed, x as i64);
    let mix = salt_mix(mix, y as i64);
    let mix = salt_mix(mix, z as i64);
    let mix = salt_mix(mix, x as i64);
    let mix = salt_mix(mix, y as i64);
    let mix = salt_mix(mix, z as i64);
    let offset_x = scale_mix(mix);
    let mix = salt_mix(mix, seed);
    let offset_y = scale_mix(mix);
    let mix = salt_mix(mix, seed);
    let offset_z = scale_mix(mix);
    square(z_part + offset_z) + square(y_part + offset_y) + square(x_part + offset_x)
}

#[inline]
fn scale_mix(l: i64) -> f64 {
    let d = floor_mod(l >> 24, 1024) as f64 / 1024.0;
    (d - 0.5) * 0.9
}

#[inline]
fn salt_mix(seed: i64, salt: i64) -> i64 {
    let mix = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    mix.wrapping_add(salt)
}
