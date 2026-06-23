use std::ops::Deref;

use atmos_primitives::reactions::ReactionRegistry;
use bevy::prelude::*;
use tile_grid::Grid;

use crate::{
    AtmosSimulated,
    chunk::ChunkMixtures,
    simulation::{AtmosSchedule, AtmosStepSystems},
    tile_mixture::TileMixtureViewMut,
};

pub(super) struct ReactionSimulation;

impl Plugin for ReactionSimulation {
    fn build(&self, app: &mut App) {
        app.add_systems(
            AtmosSchedule,
            (perform_reactions)
                .chain()
                .in_set(AtmosStepSystems::ReactionPhase),
        );
    }
}

fn do_tile_reactions(_mixture: &TileMixtureViewMut, _reactions: &ReactionRegistry) {}

fn perform_reactions(
    grids: Query<&Grid, With<AtmosSimulated>>,
    mut active_chunks: Query<&mut ChunkMixtures>,
    reactions: NonSend<ReactionRegistry>, // forces single threading btw
) {
    for grid in &grids {
        for &chunk_entity in grid.chunks.values() {
            if let Ok(mut mixtures) = active_chunks.get_mut(chunk_entity) {
                for mixture in mixtures.iter_tile_views_mut() {
                    do_tile_reactions(&mixture, reactions.deref());
                }
            }
        }
    }
}
