use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

use crate::loot::LootTable;

use super::BlockDirection;

pub static BLOCKS: LazyLock<TopLevel> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
});

pub static BLOCKS_BY_ID: LazyLock<HashMap<u16, &'static Block>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        map.insert(block.id, block);
    }
    map
});

pub static BLOCK_ID_BY_REGISTRY_ID: LazyLock<HashMap<&'static str, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        map.insert(block.name.as_str(), block.id);
    }
    map
});

pub static STATE_ID_TO_REGISTRY_ID: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        for state in &block.states {
            map.insert(state.id, block.name.as_str());
        }
    }
    map
});

pub static BLOCK_ID_BY_STATE_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        for state in &block.states {
            map.insert(state.id, block.id);
        }
    }
    map
});

pub static STATE_INDEX_BY_STATE_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        for (index, state) in block.states.iter().enumerate() {
            map.insert(state.id, index as u16);
        }
    }
    map
});

pub static BLOCK_ID_BY_ITEM_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        map.insert(block.item_id, block.id);
    }
    map
});

pub fn get_block(registry_id: &str) -> Option<&Block> {
    let key = registry_id.replace("minecraft:", "");
    let id = BLOCK_ID_BY_REGISTRY_ID.get(key.as_str())?;
    BLOCKS_BY_ID.get(id).cloned()
}

pub fn get_block_by_id<'a>(id: u16) -> Option<&'a Block> {
    BLOCKS_BY_ID.get(&id).cloned()
}

pub fn get_state_by_state_id<'a>(id: u16) -> Option<&'a State> {
    get_block_and_state_by_state_id(id).map(|(_, state)| state)
}

pub fn get_block_by_state_id<'a>(id: u16) -> Option<&'a Block> {
    let block_id = BLOCK_ID_BY_STATE_ID.get(&id)?;
    BLOCKS_BY_ID.get(block_id).cloned()
}

pub fn get_block_and_state_by_state_id<'a>(id: u16) -> Option<(&'a Block, &'a State)> {
    let block_id = BLOCK_ID_BY_STATE_ID.get(&id)?;
    let block = BLOCKS_BY_ID.get(block_id)?;
    let state_index = STATE_INDEX_BY_STATE_ID.get(&id)?;
    let state = block.states.get(*state_index as usize)?;
    Some((block, state))
}

pub fn get_block_by_item<'a>(item_id: u16) -> Option<&'a Block> {
    let block_id = BLOCK_ID_BY_ITEM_ID.get(&item_id)?;
    BLOCKS_BY_ID.get(block_id).cloned()
}

pub fn get_block_collision_shapes(block_state_id: u16) -> Option<Vec<Shape>> {
    let block = BLOCKS_BY_ID.get(&BLOCK_ID_BY_STATE_ID[&block_state_id])?;
    let state = &block.states[STATE_INDEX_BY_STATE_ID[&block_state_id] as usize];
    let mut shapes: Vec<Shape> = vec![];
    for i in 0..state.collision_shapes.len() {
        let shape = &BLOCKS.shapes[state.collision_shapes[i] as usize];
        shapes.push(shape.clone());
    }
    Some(shapes)
}

pub fn is_side_solid(shapes: &Vec<Shape>, side: BlockDirection) -> bool {
    // Define the bounds for the side we're checking
    let axis_idx = match side {
        BlockDirection::Bottom => 1, // Y-axis, min=0, max=0
        BlockDirection::Top => 1,    // Y-axis, min=1, max=1
        BlockDirection::North => 2,  // Z-axis, min=0, max=0
        BlockDirection::South => 2,  // Z-axis, min=1, max=1
        BlockDirection::West => 0,   // X-axis, min=0, max=0
        BlockDirection::East => 0,   // X-axis, min=1, max=1
    };

    let side_value = match side {
        BlockDirection::Bottom | BlockDirection::North | BlockDirection::West => 0.0,
        BlockDirection::Top | BlockDirection::South | BlockDirection::East => 1.0,
    };

    // Create a list of rectangles that cover the side
    let mut rectangles = Vec::new();

    for shape in shapes {
        // Skip shapes that don't touch the side we're checking
        let shape_min = shape.min[axis_idx];
        let shape_max = shape.max[axis_idx];

        if (side_value == 0.0 && shape_min > 0.0) || (side_value == 1.0 && shape_max < 1.0) {
            continue;
        }

        // Get the other two dimensions (not the side dimension)
        let dim1 = if axis_idx == 0 { 1 } else { 0 };
        let dim2 = if axis_idx == 2 || (axis_idx == 1 && dim1 == 0) {
            2
        } else {
            1
        };

        // Add the rectangle to our list
        if (side_value == 0.0 && shape_min == 0.0) || (side_value == 1.0 && shape_max == 1.0) {
            rectangles.push([
                shape.min[dim1],
                shape.min[dim2],
                shape.max[dim1],
                shape.max[dim2],
            ]);
        }
    }

    // Check if the rectangles cover the entire 1x1 side
    is_fully_covered(&rectangles)
}

pub fn is_solid(shapes: &Vec<Shape>) -> bool {
    shapes.iter().any(|shape| {
        shape.min[0] <= 0.0 && shape.min[1] <= 0.0 && shape.max[0] >= 1.0 && shape.max[1] >= 1.0
    })
}

// Helper function to check if a set of rectangles fully covers a 1x1 square
fn is_fully_covered(rectangles: &Vec<[f64; 4]>) -> bool {
    if rectangles.is_empty() {
        return false;
    }

    // For simplicity, we'll check if any rectangle covers the entire side
    for rect in rectangles {
        if rect[0] <= 0.0 && rect[1] <= 0.0 && rect[2] >= 1.0 && rect[3] >= 1.0 {
            return true;
        }
    }

    // A more sophisticated approach would be to check for coverage of the entire area
    // by multiple rectangles, but that's more complex and likely unnecessary for
    // most Minecraft blocks which tend to have simple shapes

    false
}

#[derive(Deserialize, Clone)]
pub struct TopLevel {
    pub block_entity_types: Vec<String>,
    shapes: Vec<Shape>,
    pub blocks: Vec<Block>,
}
#[derive(Deserialize, Clone)]
pub struct Block {
    pub id: u16,
    pub item_id: u16,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub wall_variant_id: Option<u16>,
    pub translation_key: String,
    pub name: String,
    pub loot_table: Option<LootTable>,
    pub properties: Vec<Property>,
    pub default_state_id: u16,
    pub states: Vec<State>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Property {
    pub name: String,
    pub values: Vec<String>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct State {
    pub id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub tool_required: bool,
    pub hardness: f32,
    pub opacity: Option<u32>,
    pub replaceable: bool,
    pub collision_shapes: Vec<u16>,
    pub block_entity_type: Option<u32>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Shape {
    pub min: [f64; 3],
    pub max: [f64; 3],
}
