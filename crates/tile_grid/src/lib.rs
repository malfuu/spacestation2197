//! SS 2197 Grid Solution
//! Provides 2D chunk based sparse grids, with each cell being it's own entity.
//! Alongside [`UnsizedBaseGrid`], a fixed statically sized 2D array.
//! WARNING: Currently chunk spawning is the library's user concern.
//!
//! Currently it only supports a single grid.
//! And does not despawn chunks.
//! and is overall desync happy.
//! oh god - this impl is trash!
#![deny(missing_docs)]

pub mod chunk_mask;
pub mod grid;

use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use common::TileTag;
use serde::{Deserialize, Serialize};

use crate::grid::UnsizedBaseGrid;

/// Side length of a chunk.
pub const CHUNK_SIZE: usize = 16;
/// Total area of a chunk.
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

/// Coordinates identifying a chunk's position in grid space.
pub type ChunkPosition = IVec2;
/// Coordinates identifying a tile's position in grid space.
pub type WorldTilePosition = IVec2;
/// Coordinates identifying a tile's position relative to its containing chunk.
pub type LocalTilePosition = UVec2;

/// Base grid with CHUNK_SIZE squared dimension.
/// You should mostly use this one.
pub type BaseGrid<T> = UnsizedBaseGrid<T, CHUNK_SIZE, CHUNK_SIZE, CHUNK_AREA>;
/// Sparse Chunk containing [`TileTag`]s.
pub type TileChunk = BaseGrid<Option<TileTag>>;
/// Entity per cell.
pub type EntityChunk = BaseGrid<Entity>;

/// [`BaseGrid`] with bools.
/// Avoid using this as [`chunk_mask::ChunkMask`] takes the lead.
pub type BooleanChunk = BaseGrid<bool>;

/// Chunk position origin is in the top left corner
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Chunk {
    // TODO: perhaps split this into ChunkPosition component
    position: ChunkPosition,
    /// The tile grid containing the tile tags for this chunk.
    pub tiles: TileChunk,
}

impl Chunk {
    /// Constructs a new [`Chunk`] at the specified chunk position.
    pub fn new(position: ChunkPosition) -> Self {
        Self {
            position,
            tiles: default(),
        }
    }

    /// Returns the position of this chunk.
    pub fn position(&self) -> ChunkPosition {
        self.position
    }

    /// Returns `true` if all tiles in this chunk are empty.
    pub fn is_empty(&self) -> bool {
        self.tiles.is_all_none()
    }
}

/// Grid Component
/// Addition of chunks
#[derive(Component, Serialize, Deserialize)]
#[component(map_entities)]
pub struct Grid {
    /// Internal container of entities representing chunks.
    pub chunks: HashMap<ChunkPosition, Entity>,
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl Grid {
    /// Constructs a new [`Grid`].
    pub fn new() -> Self {
        Self { chunks: default() }
    }

    /// Adds a [`Chunk`] at [`ChunkPosition`].
    pub fn add(&mut self, chunk_position: ChunkPosition, chunk: Entity) {
        assert!(self.get(chunk_position).is_none());
        self.chunks.insert(chunk_position, chunk);
    }

    /// Retrieves a [`Chunk`] indexed by [`ChunkPosition`].
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
