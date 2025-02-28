use async_trait::async_trait;
use pumpkin_macros::block_property;

use super::BlockProperty;

// Those which requires custom names to values can be defined like this
#[block_property("power", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])]
pub enum Power {
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    Level6,
    Level7,
    Level8,
    Level9,
    Level10,
    Level11,
    Level12,
    Level13,
    Level14,
    Level15,
}

#[async_trait]
impl BlockProperty for Power {}
