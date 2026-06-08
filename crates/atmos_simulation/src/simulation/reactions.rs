use bevy::prelude::*;

use crate::simulation::{AtmosSchedule, AtmosStepSystems};

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

fn perform_reactions() {
    // TODO
}
