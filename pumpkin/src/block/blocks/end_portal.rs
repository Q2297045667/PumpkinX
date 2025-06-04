use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::EntityBase;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::{Block, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::DimensionType;
use pumpkin_util::math::position::BlockPos;

#[pumpkin_block("minecraft:end_portal")]
pub struct EndPortalBlock;

#[async_trait]
impl PumpkinBlock for EndPortalBlock {
    async fn on_entity_collision(
        &self,
        world: &Arc<World>,
        entity: &dyn EntityBase,
        pos: BlockPos,
        _block: Block,
        _state: BlockState,
        server: &Server,
    ) {
        let world = if world.dimension_type == DimensionType::TheEnd {
            server
                .get_world_from_dimension(DimensionType::Overworld)
                .await
        } else {
            server.get_world_from_dimension(DimensionType::TheEnd).await
        };
        entity.get_entity().try_use_portal(0, world, pos).await;
    }
}
