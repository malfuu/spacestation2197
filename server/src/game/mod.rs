pub mod atmos;
pub mod combat;
pub mod containers;
pub mod ghost;
pub mod hands;
pub mod interact;
mod markers;
pub mod mind;
pub mod mob;
pub mod placement;
pub mod sandbox;
pub mod say;

use bevy::prelude::*;

use crate::game::{
    atmos::ServerAtmosPlugin, combat::CombatPlugin, containers::ContainersPlugin,
    ghost::GhostPlugin, hands::HandsPlugin, interact::InteractPlugin, markers::MarkersPlugin,
    mind::MindPlugin, mob::MobPlugin, placement::ServerPlacementPlugin, sandbox::SandboxPlugin,
    say::SayPlugin,
};

pub(crate) struct ServerGamePlugin;

impl Plugin for ServerGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SandboxPlugin)
            .add_plugins(SayPlugin)
            .add_plugins(ServerAtmosPlugin)
            .add_plugins(MindPlugin)
            .add_plugins(MobPlugin)
            .add_plugins(GhostPlugin)
            .add_plugins(HandsPlugin)
            .add_plugins(InteractPlugin)
            .add_plugins(MarkersPlugin)
            .add_plugins(ContainersPlugin)
            .add_plugins(ServerPlacementPlugin)
            .add_plugins(CombatPlugin);
    }
}
