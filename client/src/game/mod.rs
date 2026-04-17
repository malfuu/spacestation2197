//! Game related code.
pub mod atmos;
pub mod camera;
pub mod containers;
pub mod examine;
pub mod hands;
pub mod hud;
pub mod interact;
pub mod light;
pub mod mind;
pub mod mob;
pub mod outline;
pub mod physics;
pub mod sandbox;
pub mod say;

use bevy::prelude::*;

use crate::{
    ClientBasePlugin,
    game::{
        atmos::ClientAtmosPlugin, camera::ClientCameraPlugin, containers::ClientContainerPlugin,
        examine::ClientExaminePlugin, hands::ClientHandsPlugin, hud::ClientHudPlugin,
        interact::ClientInteractPlugin, light::ClientLightPlugin, mind::ClientMindPlugin,
        mob::ClientMobPlugin, outline::ClientOutlinePlugin, physics::ClientPhysicsPlugin,
        sandbox::ClientSandboxPlugin, say::ClientSayPlugin,
    },
};

pub(super) struct ClientGamePlugin;

impl Plugin for ClientGamePlugin {
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<ClientBasePlugin>());

        app.add_plugins(ClientOutlinePlugin)
            .add_plugins(ClientSandboxPlugin)
            .add_plugins(ClientPhysicsPlugin)
            .add_plugins(ClientHudPlugin)
            .add_plugins(ClientMindPlugin)
            .add_plugins(ClientMobPlugin)
            .add_plugins(ClientSayPlugin)
            .add_plugins(ClientInteractPlugin)
            .add_plugins(ClientHandsPlugin)
            .add_plugins(ClientAtmosPlugin)
            .add_plugins(ClientCameraPlugin)
            .add_plugins(ClientContainerPlugin)
            .add_plugins(ClientLightPlugin)
            .add_plugins(ClientExaminePlugin);
    }
}
