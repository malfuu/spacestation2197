pub mod chunk;
pub mod excited;
pub mod tile_mixture;

pub mod simulation;

#[doc(hidden)]
pub mod prelude;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::simulation::{AtmosphericsSimulationPlugin, run_atmos_schedule};

pub type AtmosTick = u32;

#[derive(Resource, Debug)]
pub struct AtmosphericsResource {
    pub enabled: bool,
    current_tick: AtmosTick,
}

pub struct AtmosphericsPlugin;

impl Plugin for AtmosphericsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AtmosphericsSimulationPlugin)
            .insert_resource(AtmosphericsResource {
                enabled: true,
                current_tick: 0,
            })
            .configure_sets(
                FixedUpdate,
                (
                    AtmosSystems::First,
                    AtmosSystems::StepSimulation,
                    AtmosSystems::Last,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                run_atmos_schedule.in_set(AtmosSystems::StepSimulation),
            );
    }
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AtmosSystems {
    /// Runs before any atmos simulation systems.
    /// Empty by default.
    First,
    /// Runs the atmospheric simulation by one step in [`crate::simulation::AtmosStepSystems`]
    /// Systems in this set are run in the [`AtmosSchedule`].
    StepSimulation,
    /// Runs after any atmos simulation systems.
    /// Empty by default.
    Last,
}

/// Marks a grid has having a simulated atmosphere.
/// Note: a grid might still have an atmosphere.
#[derive(Component, Debug, Serialize, Deserialize)]
pub struct AtmosSimulated;
