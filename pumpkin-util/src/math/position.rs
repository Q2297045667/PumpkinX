use super::vector3::Vector3;
use std::fmt;

use crate::math::vector2::Vector2;
use num_traits::Euclid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
/// Aka Block Position
pub struct BlockPos(pub Vector3<i32>);

impl BlockPos {
    pub fn chunk_and_chunk_relative_position(&self) -> (Vector2<i32>, Vector3<i32>) {
        let (z_chunk, z_rem) = self.0.z.div_rem_euclid(&16);
        let (x_chunk, x_rem) = self.0.x.div_rem_euclid(&16);
        let chunk_coordinate = Vector2 {
            x: x_chunk,
            z: z_chunk,
        };

        // Since we divide by 16 remnant can never exceed u8
        let relative = Vector3 {
            x: x_rem,
            z: z_rem,

            y: self.0.y,
        };
        (chunk_coordinate, relative)
    }
    pub fn from_i64(encoded_position: i64) -> Self {
        BlockPos(Vector3 {
            x: (encoded_position >> 38) as i32,
            y: (encoded_position << 52 >> 52) as i32,
            z: (encoded_position << 26 >> 38) as i32,
        })
    }

    pub fn offset(&self, offset: Vector3<i32>) -> Self {
        Self(self.0 + offset)
    }

    pub fn floored(x: f64, y: f64, z: f64) -> Self {
        Self(Vector3::new(
            x.floor() as i32,
            y.floor() as i32,
            z.floor() as i32,
        ))
    }

    pub fn to_f64(&self) -> Vector3<f64> {
        Vector3::new(
            self.0.x as f64 + 0.5,
            self.0.y as f64,
            self.0.z as f64 + 0.5,
        )
    }
}
impl Serialize for BlockPos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let long = ((self.0.x as i64 & 0x3FFFFFF) << 38)
            | ((self.0.z as i64 & 0x3FFFFFF) << 12)
            | (self.0.y as i64 & 0xFFF);
        serializer.serialize_i64(long)
    }
}

impl<'de> Deserialize<'de> for BlockPos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = BlockPos;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("An i64 int")
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BlockPos(Vector3 {
                    x: (v >> 38) as i32,
                    y: (v << 52 >> 52) as i32,
                    z: (v << 26 >> 38) as i32,
                }))
            }
        }
        deserializer.deserialize_i64(Visitor)
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.0.x, self.0.y, self.0.z)
    }
}
