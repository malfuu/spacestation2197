//! Meta related code.
pub mod administration;
pub mod customization;
pub mod lobby;
pub mod ooc;

use bevy::prelude::*;

use crate::{
    ClientBasePlugin, ClientGamePlugin,
    meta::{
        administration::ClientAdministrationPlugin, customization::ClientCustomizationPlugin,
        lobby::ClientLobbyPlugin, ooc::ClientOocPlugin,
    },
};

pub(super) struct ClientMetaPlugin;

impl Plugin for ClientMetaPlugin {
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<ClientBasePlugin>());
        assert!(app.is_plugin_added::<ClientGamePlugin>());

        app.add_plugins(ClientAdministrationPlugin)
            .add_plugins(ClientCustomizationPlugin)
            .add_plugins(ClientLobbyPlugin)
            .add_plugins(ClientOocPlugin);
    }
}
