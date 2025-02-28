use crate::{
    block::properties::{BlockPropertyMetadata, cardinal::North, powered::Powered},
    entity::player::Player,
};
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{
    BlockDirection,
    registry::{
        Block, State, get_block_by_state_id, get_block_collision_shapes, is_side_solid, is_solid,
    },
};

use crate::{
    block::{properties::Direction, pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    server::Server,
    world::World,
};

const WIRE_CONNECTION_EAST: usize = 0;
const WIRE_CONNECTION_NORTH: usize = 1;
const WIRE_CONNECTION_POWER_LEVEL: usize = 2;
const WIRE_CONNECTION_SOUTH: usize = 3;
const WIRE_CONNECTION_WEST: usize = 4;
const REPEATER_FACING: usize = 1;
const OBSERVER_FACING: usize = 0;

pub fn is_connected(state: &str) -> bool {
    state != North::None.value()
}

pub fn is_fully_connected(states: Vec<String>) -> bool {
    return is_connected(states[WIRE_CONNECTION_EAST].as_str())
        && is_connected(states[WIRE_CONNECTION_NORTH].as_str())
        && is_connected(states[WIRE_CONNECTION_SOUTH].as_str())
        && is_connected(states[WIRE_CONNECTION_WEST].as_str());
}

pub fn is_not_connected(states: Vec<String>) -> bool {
    return !is_connected(states[WIRE_CONNECTION_EAST].as_str())
        && !is_connected(states[WIRE_CONNECTION_NORTH].as_str())
        && !is_connected(states[WIRE_CONNECTION_SOUTH].as_str())
        && !is_connected(states[WIRE_CONNECTION_WEST].as_str());
}

pub async fn connects_to(
    state: &State,
    direction: Option<BlockDirection>,
    server: &Server,
) -> bool {
    let block = get_block_by_state_id(state.id).unwrap();

    if block.name == "redstone_wire" {
        return true;
    } else if block.name == "repeater" {
        if let Some(direction) = direction {
            let repeater_state = server
                .block_properties_manager
                .get_states(block, state)
                .await;
            let facing = BlockDirection::try_from(repeater_state[REPEATER_FACING].as_str())
                .unwrap_or(BlockDirection::North);
            return facing == direction || facing == direction.opposite();
        }
    } else if block.name == "observer" {
        if let Some(direction) = direction {
            let observer_state = server
                .block_properties_manager
                .get_states(block, state)
                .await;
            let facing = BlockDirection::try_from(observer_state[OBSERVER_FACING].as_str())
                .unwrap_or(BlockDirection::North);
            return facing == direction;
        }
    } else if let Some(pumpkin_block) = server.block_registry.get_pumpkin_block(block) {
        return pumpkin_block.emits_redstone_power(state) && direction.is_some();
    }

    false
}

pub async fn get_render_connection_type(
    world: &World,
    location: BlockPos,
    direction: BlockDirection,
    is_not_solid: bool,
    server: &Server,
) -> North {
    let other_block_pos = location.offset(direction.to_offset());
    let other_block = world.get_block(&other_block_pos).await.unwrap();
    let other_block_state = world.get_block_state(&other_block_pos).await.unwrap();

    if is_not_solid {
        let is_trapdoor = other_block.name.contains("trapdoor");
        let can_run_on_top = can_run_on_top(other_block_state);
        if is_trapdoor && can_run_on_top {
            return North::Up;
        }
    }

    if connects_to(other_block_state, Some(direction), server).await
        && can_run_on_top(other_block_state)
        || !connects_to(
            world
                .get_block_state(&other_block_pos.offset(BlockDirection::Bottom.to_offset()))
                .await
                .unwrap(),
            None,
            server,
        )
        .await
    {
        return North::None;
    }

    North::Side
}

pub fn can_run_on_top(floor: &State) -> bool {
    is_solid(&get_block_collision_shapes(floor.id).unwrap())
}

//TODO: Use item tag or something here
#[pumpkin_block("minecraft:redstone_wire")]
pub struct RedstoneWireBlock;

#[async_trait]
impl PumpkinBlock for RedstoneWireBlock {
    async fn can_place_on_side(
        &self,
        world: &World,
        location: BlockPos,
        _side: BlockDirection,
    ) -> bool {
        let target_block_pos = BlockPos(location.0 + BlockDirection::Bottom.to_offset());
        can_run_on_top(world.get_block_state(&target_block_pos).await.unwrap())
            || world.get_block(&target_block_pos).await.unwrap().name == "hopper"
    }

    async fn normal_use(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &World,
    ) {
        let collision_shapes = get_block_collision_shapes(4769);
        println!("Collision shapes: {:?}", collision_shapes);
        println!("Redstone wire used");
    }

    async fn use_with_item(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _item: &Item,
        _server: &Server,
        _world: &World,
    ) -> BlockActionResult {
        println!("Redstone wire used");
        BlockActionResult::Consume
    }
}
