//! SS 2197 Grid Solution
//! Provides 2D chunk based sparse grids, with each cell being it's own entity.
//! Alongside [`UnsizedBaseGrid`], a fixed statically sized 2D array.
//! WARNING: Currently chunk spawning is the library's user concern.
//!
//! Currently it only supports a single grid.
//! And does not despawn chunks.
//! and is overall desync happy.
//! oh god - this impl is trash!
pub mod chunk_mask;
pub mod grid;

use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use common::TileTag;
use serde::{Deserialize, Serialize};

use crate::grid::UnsizedBaseGrid;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

pub type ChunkPosition = IVec2;
pub type WorldTilePosition = IVec2;
pub type LocalTilePosition = UVec2;

/// Grid with CHUNK_SIZE squared dimension.
pub type BaseGrid<T> = UnsizedBaseGrid<T, CHUNK_SIZE, CHUNK_SIZE, CHUNK_AREA>;
pub type TileChunk = BaseGrid<Option<TileTag>>;
pub type EntityChunk = BaseGrid<Entity>;

pub type BooleanChunk = BaseGrid<bool>; // perhaps look into fitting this inside a u64 bitmask
// actually look into usin the new [`chunkMask`]!

/// Chunk position origin is in the top left corner
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Chunk {
    position: ChunkPosition, // TODO: perhaps split this into ChunkPosition component
    pub tiles: TileChunk,
}

impl Chunk {
    pub fn new(position: ChunkPosition) -> Self {
        Self {
            position,
            tiles: default(),
        }
    }

    pub fn position(&self) -> ChunkPosition {
        self.position
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_all_none()
    }
}

/// Grid Component
/// Addition of chunks
#[derive(Component, Serialize, Deserialize)]
#[component(map_entities)]
pub struct Grid {
    pub chunks: HashMap<ChunkPosition, Entity>,
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl Grid {
    pub fn new() -> Self {
        Self { chunks: default() }
    }

    pub fn add(&mut self, chunk_position: ChunkPosition, chunk: Entity) {
        assert!(self.get(chunk_position).is_none());
        self.chunks.insert(chunk_position, chunk);
    }

    pub fn get(&self, chunk_position: ChunkPosition) -> Option<Entity> {
        self.chunks.get(&chunk_position).copied()
    }
}

impl MapEntities for Grid {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for (_, chunk_entity) in self.chunks.iter_mut() {
            *chunk_entity = entity_mapper.get_mapped(*chunk_entity);
        }
    }
}

/// Transforms a world position to a chunk position in accordance to [`CHUNK_SIZE`]
/// and a relative local position.
#[inline]
pub fn world_to_chunk_and_local(
    world_position: WorldTilePosition,
) -> (ChunkPosition, LocalTilePosition) {
    let chunk_position =
        world_position.div_euclid(IVec2::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32));
    let local_position = world_position
        .rem_euclid(IVec2::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32))
        .as_uvec2();

    (chunk_position, local_position)
}

/// Helper function to transform chunk position and relative local position into a world position.
#[inline]
pub fn chunk_and_local_to_world(
    chunk_position: ChunkPosition,
    local_position: LocalTilePosition,
) -> WorldTilePosition {
    let chunk_size = IVec2::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32);
    chunk_position * chunk_size + IVec2::new(local_position.x as i32, local_position.y as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin() {
        let (chunk, offset) = world_to_chunk_and_local(WorldTilePosition::new(0, 0));
        assert_eq!(chunk, ChunkPosition::new(0, 0));
        assert_eq!(offset, LocalTilePosition::new(0, 0));
    }

    #[test]
    fn test_within_first_chunk() {
        let (chunk, offset) = world_to_chunk_and_local(WorldTilePosition::new(7, 7));
        assert_eq!(chunk, ChunkPosition::new(0, 0));
        assert_eq!(offset, LocalTilePosition::new(7, 7));
    }

    #[test]
    fn test_next_chunk_boundary() {
        let (chunk, offset) = world_to_chunk_and_local(WorldTilePosition::new(16, 16));
        assert_eq!(chunk, ChunkPosition::new(1, 1));
        assert_eq!(offset, LocalTilePosition::new(0, 0));
    }

    #[test]
    fn test_negative_coords_simple() {
        let (chunk, offset) = world_to_chunk_and_local(WorldTilePosition::new(-1, 0));

        assert_eq!(chunk, ChunkPosition::new(-1, 0));
        assert_eq!(offset, LocalTilePosition::new(15, 0));
    }
}
