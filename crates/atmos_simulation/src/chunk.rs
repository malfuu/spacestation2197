use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use grid::{BaseGrid, BooleanChunk, CHUNK_SIZE, grid::UnsizedBaseGrid};

use atmos_primitives::prelude::*;

use crate::active::ProcessedTick;

pub type MixtureChunk = BaseGrid<BasicGasMixture>;
pub type FlowChunk = BaseGrid<Vec2>;

#[derive(Component, Deref, DerefMut, Default, Serialize, Deserialize)]
pub struct Flows(pub FlowChunk);

/// Gas Mixtures per tile.
#[derive(Component, Serialize, Deserialize)]
#[require(ChunkDeltas, Flows, SpaceChunk, ImpermeableChunk, ProcessedTick)]
pub struct Mixtures {
    pub(crate) mixtures: MixtureChunk,
    // pub(crate) flows: BaseGrid<Vec2>,
}

impl Default for Mixtures {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixtures {
    pub fn new() -> Self {
        let tile_gas_mixture = BasicGasMixture::new_empty(2.5);
        Self {
            mixtures: BaseGrid::from_value(tile_gas_mixture),
        }
    }

    pub fn mixtures(&self) -> &MixtureChunk {
        &self.mixtures
    }

    pub fn mixtures_mut(&mut self) -> &mut MixtureChunk {
        &mut self.mixtures
    }
}

/// Differences in pressures (Pascals) between tiles.
pub(crate) type Delta = PressureArray;
pub(crate) type InterchunkDeltas = [Delta; CHUNK_SIZE];

/// Length per delta axis. Since deltas are the edges between cells in a grid,
/// this will reduce the amount of them by 1.
pub const DELTAS_LENGTH: usize = CHUNK_SIZE - 1;
pub const DELTAS_AREA: usize = DELTAS_LENGTH * CHUNK_SIZE;

#[derive(Component, Default, Clone, Copy)]
pub(crate) struct ChunkDeltas {
    pub(crate) horizontals: UnsizedBaseGrid<Delta, CHUNK_SIZE, DELTAS_LENGTH, DELTAS_AREA>,
    pub(crate) verticals: UnsizedBaseGrid<Delta, DELTAS_LENGTH, CHUNK_SIZE, DELTAS_AREA>,

    // because I cant express deltas in between sparse chunks in a clean data structure
    // each chunk will contain deltas to neighboring chunks in itself.
    // this of course will introduce duplicated data for two neighboring chunks
    // yipee!!
    pub(crate) left: InterchunkDeltas,
    pub(crate) right: InterchunkDeltas,
    pub(crate) up: InterchunkDeltas,
    pub(crate) down: InterchunkDeltas,
}

#[derive(Component, Clone, Copy, Deref, DerefMut)]
pub struct SpaceChunk(pub BooleanChunk);

impl Default for SpaceChunk {
    fn default() -> Self {
        Self(BooleanChunk::from_value(true))
    }
}

#[derive(Component, Default, Clone, Copy, Deref, DerefMut)]
pub struct ImpermeableChunk(pub BooleanChunk);
