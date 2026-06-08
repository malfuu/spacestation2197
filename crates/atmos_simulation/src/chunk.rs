use std::array;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use tile_grid::{BaseGrid, BooleanChunk, CHUNK_SIZE, LocalTilePosition};

use crate::tile_mixture::{CachedTile, TileEnergy, TileMixtureView, TileMixtureViewMut, TileMoles};

#[derive(Component, Default, Clone, Copy, Deref, DerefMut)]
pub struct ChunkCached(pub BaseGrid<CachedTile>);

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
    pub fn tile_view(&self, pos: LocalTilePosition) -> Option<TileMixtureView<'_>> {
        let moles = self.moles.0.get(pos)?;
        let energy = self.energy.0.get(pos)?;
        Some(TileMixtureView::new(moles, energy))
    }

    pub fn tile_view_mut(&mut self, pos: LocalTilePosition) -> Option<TileMixtureViewMut<'_>> {
        let moles = self.moles.0.get_mut(pos)?;
        let energy = self.energy.0.get_mut(pos)?;
        Some(TileMixtureViewMut::new(moles, energy))
    }

    pub fn tile_view_many<const N: usize>(
        &self,
        positions: [LocalTilePosition; N],
    ) -> Option<[TileMixtureView<'_>; N]> {
        let moles = self.moles.0.get_many(positions)?;
        let energies = self.energy.0.get_many(positions)?;

        Some(array::from_fn(|i| {
            TileMixtureView::new(moles[i], energies[i])
        }))
    }

    pub fn tile_view_many_mut<const N: usize>(
        &mut self,
        positions: [LocalTilePosition; N],
    ) -> Option<[TileMixtureViewMut<'_>; N]> {
        let moles = self.moles.0.get_many_mut(positions)?;
        let energies = self.energy.0.get_many_mut(positions)?;

        let mut zipped = core::iter::zip(moles, energies);

        Some(array::from_fn(|_| {
            let (m, e) = zipped.next().unwrap();
            TileMixtureViewMut::new(m, e)
        }))
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

pub type FlowChunk = BaseGrid<Vec2>;

#[derive(Component, Deref, DerefMut, Default, Serialize, Deserialize)]
pub struct Flows(pub FlowChunk);

/// Internal edges in a chunk length. Mostly used when iterating and comparing between cells.
pub const INTERNAL_EDGES_LENGTH: usize = CHUNK_SIZE - 1;

#[derive(Component, Clone, Copy, Deref, DerefMut)]
pub struct SpaceChunk(pub BooleanChunk);

impl Default for SpaceChunk {
    fn default() -> Self {
        Self(BooleanChunk::from_value(true))
    }
}

#[derive(Component, Default, Clone, Copy, Deref, DerefMut)]
pub struct ImpermeableChunk(pub BooleanChunk);
