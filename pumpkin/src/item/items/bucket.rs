use std::sync::Arc;

use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::{Block, BlockState, fluid::Fluid, item::Item};
use pumpkin_registry::DimensionType;
use pumpkin_util::{GameMode, math::position::BlockPos};
use pumpkin_world::{inventory::Inventory, item::ItemStack, world::BlockFlags};
use pumpkin_util::math::vector3::Vector3;

use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::world::World;

pub struct EmptyBucketItem;
pub struct FilledBucketItem;

impl ItemMetadata for EmptyBucketItem {
    fn ids() -> Box<[u16]> {
        [Item::BUCKET.id].into()
    }
}

impl ItemMetadata for FilledBucketItem {
    fn ids() -> Box<[u16]> {
        [
            Item::WATER_BUCKET.id,
            Item::LAVA_BUCKET.id,
            // TODO drink milk
            // Item::MILK_BUCKET.id,
            // TODO implement these buckets, and getting the item from the world
            // Item::POWDER_SNOW_BUCKET.id,
            // Item::AXOLOTL_BUCKET.id,
            // Item::COD_BUCKET.id,
            // Item::SALMON_BUCKET.id,
            // Item::TROPICAL_FISH_BUCKET.id,
            // Item::PUFFERFISH_BUCKET.id,
            // Item::TADPOLE_BUCKET.id,
        ]
        .into()
    }
}

impl ItemMetadata for MilkBucketItem {
    fn ids() -> Box<[u16]> {
        [Item::MILK_BUCKET.id].into()
    }
}

fn waterlogged_check(block: &Block, state: &BlockState) -> Option<bool> {
    block.properties(state.id).and_then(|properties| {
        properties
            .to_props()
            .into_iter()
            .find(|p| p.0 == "waterlogged")
            .map(|(_, value)| value == true.to_string())
    })
}

fn set_waterlogged(block: &Block, state: &BlockState, waterlogged: bool) -> u16 {
    let original_props = &block.properties(state.id).unwrap().to_props();
    let mut props_vec: Vec<(&str, &str)> = Vec::with_capacity(original_props.len());
    let waterlogged = waterlogged.to_string();
    for (key, value) in original_props {
        if key == "waterlogged" {
            props_vec.push((key.as_str(), &waterlogged));
        } else {
            props_vec.push((key.as_str(), value.as_str()));
        }
    }
    block.from_properties(props_vec).unwrap().to_state_id(block)
}

#[async_trait]
impl PumpkinItem for EmptyBucketItem {
    async fn normal_use(&self, _item: &Item, player: &Player) {
        let world = player.world().await.clone();
        let (start_pos, end_pos) = self.get_start_and_end_pos(player);

        let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
            let state_id = world_inner.get_block_state_id(pos).await;

            state_id == Block::WATER.default_state_id || state_id == Block::LAVA.default_state_id
        };

        let (block_pos, _) = world.raytrace(start_pos, end_pos, checker).await;

        if let Some(pos) = block_pos {
            world
                .set_block_state(&pos, Block::AIR.id, BlockFlags::NOTIFY_NEIGHBORS)
                .await;
            //TODO: Pickup in inv
        }
    }
}

#[async_trait]
impl PumpkinItem for FilledBucketItem {
    async fn normal_use(&self, item: &Item, player: &Player) {
        if item.id == Item::MILK_BUCKET.id {
            // TODO implement milk bucket
            return;
        }

        let world = player.world().await.clone();
        let (start_pos, end_pos) = self.get_start_and_end_pos(player);
        let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
            let state_id = world_inner.get_block_state_id(pos).await;
            if Fluid::from_state_id(state_id).is_some() {
                return false;
            }
            state_id != Block::AIR.id
        };

        let (block_pos, block_direction) = world.raytrace(start_pos, end_pos, checker).await;

        if let (Some(pos), Some(direction)) = (block_pos, block_direction) {
            world
                .set_block_state(
                    &pos.offset(direction.to_offset()),
                    // Block::WATER.default_state_id,
                    if item.id == Item::WATER_BUCKET.id {
                        Block::WATER.default_state_id
                    } else {
                        Block::LAVA.default_state_id
                    },
                    BlockFlags::NOTIFY_NEIGHBORS,
                )
                .await;
            if player.gamemode.load() != GameMode::Creative {
                //TODO: Pickup in inv
            }
        }
    }
}
