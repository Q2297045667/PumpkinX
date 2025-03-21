use pumpkin_nbt::nbt_long_array;
use pumpkin_util::math::vector2::Vector2;
use serde::{Deserialize, Serialize};
use std::iter::repeat_with;
use thiserror::Error;

use crate::coordinates::ChunkRelativeBlockCoordinates;

pub mod format;
pub mod io;

pub const CHUNK_WIDTH: usize = 16;
pub const WORLD_HEIGHT: usize = 384;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const SUBCHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_WIDTH;
pub const SUBCHUNKS_COUNT: usize = WORLD_HEIGHT / CHUNK_WIDTH;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * WORLD_HEIGHT;

#[derive(Error, Debug)]
pub enum ChunkReadingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Invalid header")]
    InvalidHeader,
    #[error("Region is invalid")]
    RegionIsInvalid,
    #[error("Compression error {0}")]
    Compression(CompressionError),
    #[error("Tried to read chunk which does not exist")]
    ChunkNotExist,
    #[error("Failed to parse Chunk from bytes: {0}")]
    ParsingError(ChunkParsingError),
}

#[derive(Error, Debug)]
pub enum ChunkWritingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Compression error {0}")]
    Compression(CompressionError),
    #[error("Chunk serializing error: {0}")]
    ChunkSerializingError(String),
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Compression scheme not recognised")]
    UnknownCompression,
    #[error("Error while working with zlib compression: {0}")]
    ZlibError(std::io::Error),
    #[error("Error while working with Gzip compression: {0}")]
    GZipError(std::io::Error),
    #[error("Error while working with LZ4 compression: {0}")]
    LZ4Error(std::io::Error),
    #[error("Error while working with zstd compression: {0}")]
    ZstdError(std::io::Error),
}

#[derive(Clone)]
pub struct ChunkData {
    /// See description in `Subchunks`
    pub subchunks: Subchunks,
    /// See `https://minecraft.wiki/w/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
    pub position: Vector2<i32>,
}

/// # Subchunks
/// Subchunks - its an areas in chunk, what are 16 blocks in height.
/// Current amount is 24.
///
/// Subchunks can be single and multi.
///
/// Single means a single block in all chunk, like
/// chunk, what filled only air or only water.
///
/// Multi means a normal chunk, what contains 24 subchunks.
#[derive(PartialEq, Debug, Clone)]
pub enum Subchunks {
    Single(u16),
    Multi(Box<[Subchunk; SUBCHUNKS_COUNT]>),
}

/// # Subchunk
/// Subchunk - its an area in chunk, what are 16 blocks in height
///
/// Subchunk can be single and multi.
///
/// Single means a single block in all subchunk, like
/// subchunk, what filled only air or only water.
///
/// Multi means a normal subchunk, what contains 4096 blocks.
#[derive(Clone, PartialEq, Debug)]
pub enum Subchunk {
    Single(u16),
    // The packet relies on this ordering -> leave it like this for performance
    /// Ordering: yzx (y being the most significant)
    Multi(Box<[u16; SUBCHUNK_VOLUME]>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    #[serde(serialize_with = "nbt_long_array")]
    motion_blocking: Box<[i64]>,
    #[serde(serialize_with = "nbt_long_array")]
    world_surface: Box<[i64]>,
}

/// The Heightmap for a completely empty chunk
impl Default for ChunkHeightmaps {
    fn default() -> Self {
        Self {
            // 0 packed into an i64 7 times.
            motion_blocking: vec![0; 37].into_boxed_slice(),
            world_surface: vec![0; 37].into_boxed_slice(),
        }
    }
}

impl Subchunk {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Single(block) => Some(*block),
            Self::Multi(blocks) => blocks.get(convert_index(position)).copied(),
        }
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id)
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        new_block: u16,
    ) {
        match self {
            Self::Single(block) => {
                if *block != new_block {
                    let mut blocks = Box::new([*block; SUBCHUNK_VOLUME]);
                    blocks[convert_index(position)] = new_block;

                    *self = Self::Multi(blocks)
                }
            }
            Self::Multi(blocks) => {
                blocks[convert_index(position)] = new_block;

                if blocks.iter().all(|b| *b == new_block) {
                    *self = Self::Single(new_block)
                }
            }
        }
    }

    pub fn clone_as_array(&self) -> Box<[u16; SUBCHUNK_VOLUME]> {
        match &self {
            Self::Single(block) => Box::new([*block; SUBCHUNK_VOLUME]),
            Self::Multi(blocks) => blocks.clone(),
        }
    }
}

impl Subchunks {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Single(block) => Some(*block),
            Self::Multi(subchunks) => subchunks
                .get(position.y.get_absolute() as usize / CHUNK_WIDTH)
                .and_then(|subchunk| subchunk.get_block(position)),
        }
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id)
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        new_block: u16,
    ) {
        match self {
            Self::Single(block) => {
                if *block != new_block {
                    let mut subchunks = vec![Subchunk::Single(0); SUBCHUNKS_COUNT];

                    subchunks[position.y.get_absolute() as usize / CHUNK_WIDTH]
                        .set_block(position, new_block);

                    *self = Self::Multi(subchunks.try_into().unwrap());
                }
            }
            Self::Multi(subchunks) => {
                subchunks[position.y.get_absolute() as usize / CHUNK_WIDTH]
                    .set_block(position, new_block);

                if subchunks
                    .iter()
                    .all(|subchunk| *subchunk == Subchunk::Single(new_block))
                {
                    *self = Self::Single(new_block)
                }
            }
        }
    }

    //TODO: Needs optimizations
    pub fn array_iter(&self) -> Box<dyn Iterator<Item = Box<[u16; SUBCHUNK_VOLUME]>> + '_> {
        match self {
            Self::Single(block) => {
                Box::new(repeat_with(|| Box::new([*block; SUBCHUNK_VOLUME])).take(SUBCHUNKS_COUNT))
            }
            Self::Multi(blocks) => {
                Box::new(blocks.iter().map(|subchunk| subchunk.clone_as_array()))
            }
        }
    }
}

impl ChunkData {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        self.subchunks.get_block(position)
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.subchunks.set_block(position, block_id);
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        block: u16,
    ) {
        self.subchunks
            .set_block_no_heightmap_update(position, block);
    }

    #[expect(dead_code)]
    fn calculate_heightmap(&self) -> ChunkHeightmaps {
        // figure out how LongArray is formatted
        // figure out how to find out if block is motion blocking
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum ChunkParsingError {
    #[error("Failed reading chunk status {0}")]
    FailedReadStatus(pumpkin_nbt::Error),
    #[error("The chunk isn't generated yet")]
    ChunkNotGenerated,
    #[error("Error deserializing chunk: {0}")]
    ErrorDeserializingChunk(String),
}

fn convert_index(index: ChunkRelativeBlockCoordinates) -> usize {
    // % works for negative numbers as intended.
    (index.y.get_absolute() as usize % CHUNK_WIDTH) * CHUNK_AREA
        + *index.z as usize * CHUNK_WIDTH
        + *index.x as usize
}
#[derive(Error, Debug)]
pub enum ChunkSerializingError {
    #[error("Error serializing chunk: {0}")]
    ErrorSerializingChunk(pumpkin_nbt::Error),
}
