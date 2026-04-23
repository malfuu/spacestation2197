use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use tile_grid::{BaseGrid, BooleanChunk, CHUNK_SIZE, grid::UnsizedBaseGrid};

use atmos_primitives::prelude::*;

use crate::tile_mixture::{TileEnergy, TileMixtureView, TileMixtureViewMut, TileMoles};

#[derive(Default, Serialize, Deserialize)]
pub struct ChunkMoles(pub BaseGrid<TileMoles>);

#[derive(Default, Serialize, Deserialize)]
pub struct ChunkEnergy(pub BaseGrid<TileEnergy>);

#[derive(Component, Default, Serialize, Deserialize)]
pub struct ChunkMixtures {
    moles: ChunkMoles,
    energy: ChunkEnergy,
}

impl ChunkMixtures {
    pub fn tile_view(&self, pos: UVec2) -> Option<TileMixtureView<'_>> {
        let moles = self.moles.0.get(pos)?;
        let energy = self.energy.0.get(pos)?;
        Some(TileMixtureView::new(moles, energy))
    }

    pub fn tile_view_two(
        &self,
        pos_a: UVec2,
        pos_b: UVec2,
    ) -> Option<(TileMixtureView<'_>, TileMixtureView<'_>)> {
        let (moles_a, moles_b) = self.moles.0.get_two(pos_a, pos_b)?;
        let (energy_a, energy_b) = self.energy.0.get_two(pos_a, pos_b)?;
        Some((
            TileMixtureView::new(moles_a, energy_a),
            TileMixtureView::new(moles_b, energy_b),
        ))
    }

    pub fn tile_view_mut(&mut self, pos: UVec2) -> Option<TileMixtureViewMut<'_>> {
        let moles = self.moles.0.get_mut(pos)?;
        let energy = self.energy.0.get_mut(pos)?;
        Some(TileMixtureViewMut::new(moles, energy))
    }

    pub fn tile_view_two_mut(
        &mut self,
        pos_a: UVec2,
        pos_b: UVec2,
    ) -> Option<(TileMixtureViewMut<'_>, TileMixtureViewMut<'_>)> {
        let (moles_a, moles_b) = self.moles.0.get_two_mut(pos_a, pos_b)?;
        let (energy_a, energy_b) = self.energy.0.get_two_mut(pos_a, pos_b)?;
        Some((
            TileMixtureViewMut::new(moles_a, energy_a),
            TileMixtureViewMut::new(moles_b, energy_b),
        ))
    }

    pub fn iter_tile_views_mut(&mut self) -> impl Iterator<Item = TileMixtureViewMut<'_>> {
        self.moles
            .0
            .iter_mut()
            .zip(self.energy.0.iter_mut())
            .map(|(moles, energy)| TileMixtureViewMut::new(moles, energy))
    }

    pub fn cull(&mut self) {
        self.iter_tile_views_mut().for_each(|mut m| m.cull());
    }
}

// pub type MixtureChunk = BaseGrid<BasicGasMixture>;
pub type FlowChunk = BaseGrid<Vec2>;

#[derive(Component, Deref, DerefMut, Default, Serialize, Deserialize)]
pub struct Flows(pub FlowChunk);

/// Differences in pressures (Pascals) between tiles.
pub(crate) type Delta = PressureArray;
pub(crate) type InterchunkDeltas = [Delta; CHUNK_SIZE];

/// Length per delta axis. Since deltas are the edges between cells in a grid,
/// this will reduce the amount of them by 1.
pub const DELTAS_LENGTH: usize = CHUNK_SIZE - 1;
pub const DELTAS_AREA: usize = DELTAS_LENGTH * CHUNK_SIZE;

#[derive(Component, Default, Clone, Copy)]
pub struct ChunkDeltas {
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
