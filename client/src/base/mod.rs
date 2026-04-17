//! Basic foundation building blocks required by the client
//! e.g. states, audio, input, windows, ui, main menu stuff, etc...
// I will admit, there could be a better name for this module.
pub mod audio;
pub mod camera;
pub mod chatbox;
pub mod entities;
pub mod grid;
pub mod input;
pub mod menus;
pub mod placement;
pub mod session;
pub mod settings;
pub mod states;
pub mod windows;

use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::base::{
    audio::ClientAudioPlugin, camera::ClientCameraPlugin, chatbox::ClientChatboxPlugin,
    entities::ClientEntityPlugin, grid::ClientGridPlugin, input::ClientInputPlugin,
    menus::ClientMenusPlugin, placement::ClientPlacementPlugin, session::ClientSessionPlugin,
    settings::ClientSettingsPlugin, states::ClientStatesPlugin, windows::ClientWindowsPlugin,
};

pub(super) struct ClientBasePlugin;

impl Plugin for ClientBasePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(ClientAudioPlugin)
            .add_plugins(ClientInputPlugin)
            .add_plugins(ClientCameraPlugin)
            .add_plugins(ClientSettingsPlugin)
            .add_plugins(ClientSessionPlugin)
            .add_plugins(ClientStatesPlugin)
            .add_plugins(ClientMenusPlugin)
            .add_plugins(ClientWindowsPlugin)
            .add_plugins(ClientEntityPlugin)
            .add_plugins(ClientGridPlugin)
            .add_plugins(ClientChatboxPlugin)
            .add_plugins(ClientPlacementPlugin);
    }
}
