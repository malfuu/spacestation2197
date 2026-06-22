use bevy::prelude::*;

use crate::{
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

fn perform_reactions(mut active_chunks: Query<&mut ChunkMixtures>) {
    for mut _chunk in active_chunks.iter_mut() {
        // TODO
    }
}
