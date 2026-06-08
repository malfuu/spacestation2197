pub(crate) mod flow;
pub(crate) mod reactions;

use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::{
    AtmosphericsResource,
    simulation::{flow::FlowSimulation, reactions::ReactionSimulation},
};

pub(super) struct AtmosphericsSimulationPlugin;

impl Plugin for AtmosphericsSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(AtmosSchedule)
            .add_plugins(FlowSimulation)
            .add_plugins(ReactionSimulation)
            .configure_sets(
                AtmosSchedule,
                (
                    AtmosStepSystems::First,
                    AtmosStepSystems::FlowPhase,
                    AtmosStepSystems::ReactionPhase,
                    AtmosStepSystems::Last,
                )
                    .chain(),
            );
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct AtmosSchedule;

pub(crate) fn run_atmos_schedule(world: &mut World) {
    let _ = info_span!("run_atmos_schedule", name = "atmos").entered();

    let mut r = world.resource_mut::<AtmosphericsResource>();
    if !r.enabled {
        return;
    }

    r.current_tick += 1;

    world.run_schedule(AtmosSchedule);
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AtmosStepSystems {
    /// Runs at the start of the [`AtmosSchedule`].
    First,
    /// Responsible for executing gas exchanges between environmental mixtures.
    FlowPhase,
    /// Responsible for performing reactions in environmental mixtures.
    ReactionPhase,
    /// Runs at the end of [`AtmosSchedule`]
    Last,
}
