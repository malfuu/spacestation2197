use bevy::prelude::*;

use crate::simulation::{AtmosSchedule, AtmosStepSystems};

pub(super) struct HotspotSimulation;

impl Plugin for HotspotSimulation {
    fn build(&self, app: &mut App) {
        app.add_systems(
            AtmosSchedule,
            (update_hotspots)
                .chain()
                .in_set(AtmosStepSystems::HotspotPhase),
        );
    }
}

fn update_hotspots() {}
