use bevy::prelude::*;
use tile_grid::Grid;

use crate::{
    AtmosSimulated,
    chunk::ChunkMixtures,
    simulation::{AtmosSchedule, AtmosStepSystems},
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

fn perform_reactions(
    grids: Query<&Grid, With<AtmosSimulated>>,
    mut active_chunks: Query<&mut ChunkMixtures>,
) {
    for grid in &grids {
        for &chunk_entity in grid.chunks.values() {
            if let Ok(mut _chunk) = active_chunks.get_mut(chunk_entity) {
                // TODO
            }
        }
    }
}
