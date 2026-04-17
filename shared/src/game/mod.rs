pub mod atmos;
pub mod combat;
pub mod containers;
pub mod ghost;
pub mod grid;
pub mod hands;
pub mod interact;
pub mod items;
pub mod light;
pub mod machinery;
pub mod markers;
pub mod mind;
pub mod mob;
pub mod persistence;
pub mod physics;
pub mod placement;
pub mod player;
pub mod power;
pub mod sandbox;
pub mod say;

use bevy::prelude::*;

use content::prelude::*;

use crate::{
    audio::AudioPlugin,
    game::{
        atmos::AtmosPlugin, combat::CombatPlugin, containers::ContainersPlugin, ghost::GhostPlugin,
        grid::GridPlugin, hands::HandsPlugin, interact::InteractPlugin, items::ItemPlugin,
        light::LightPlugin, machinery::MachineryPlugin, markers::MarkerPlugin, mind::MindPlugin,
        mob::MobPlugin, persistence::PersistencePlugin, placement::PlacementPlugin,
        player::PlayerPlugin, power::PowerPlugin, sandbox::SandboxPlugin, say::SayPlugin,
    },
};

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<ContentPlugin>());
        assert!(app.is_plugin_added::<AudioPlugin>());

        app.configure_sets(
            FixedUpdate,
            (
                GameplaySystems::Inputs,
                GameplaySystems::Logic,
                GameplaySystems::Final,
            )
                .chain(), // TODO: add run_if condition for toggling gameplay
        )
        .insert_resource(Time::<Fixed>::from_hz(30.0))
        .add_plugins(avian3d::PhysicsPlugins::default())
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(AtmosPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(PowerPlugin)
        .add_plugins(ContainersPlugin)
        .add_plugins(PlacementPlugin)
        .add_plugins(ItemPlugin)
        .add_plugins(LightPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(SandboxPlugin)
        .add_plugins(MobPlugin)
        .add_plugins(MindPlugin)
        .add_plugins(GhostPlugin)
        .add_plugins(SayPlugin)
        .add_plugins(HandsPlugin)
        .add_plugins(InteractPlugin) // not to be confused with bevy's InteractionPlugin
        .add_plugins(CombatPlugin)
        .add_plugins(MarkerPlugin)
        .add_plugins(MachineryPlugin)
        .add_plugins(PersistencePlugin);
    }
}

/// Runs all gameplay related stuff e.g. atmos, gameplay, mobsay etc...
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySystems {
    /// Receives player inputs
    Inputs,
    /// Normal systems here
    Logic,
    /// Finalizing and sending messages to the client
    Final,
}
