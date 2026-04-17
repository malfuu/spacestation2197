#![allow(
    dead_code,
    unused_mut,
    unused_variables,
    reason = "Needed flexibility on game programming."
)]
pub mod base;
pub mod game;
pub mod meta;

mod editor;

#[cfg(debug_assertions)]
mod debug_tools;

mod placeholder;

#[cfg(debug_assertions)]
use crate::debug_tools::ClientDebugPlugin;

use bevy::{prelude::*, window::PresentMode};

use shared::SharedPlugin;

use crate::{
    base::ClientBasePlugin, editor::ClientEditorPlugin, game::ClientGamePlugin,
    meta::ClientMetaPlugin, placeholder::ClientPlaceholderPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            // funny, how no one will complain if vsync is off
            // but many will complain if vsync is on.
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(SharedPlugin)
    // .add_plugins(ServerPlugin) TODO: turn client into possible host as well
    .add_plugins(ClientBasePlugin)
    .add_plugins(ClientEditorPlugin)
    .add_plugins(ClientGamePlugin)
    .add_plugins(ClientMetaPlugin)
    .add_plugins(ClientPlaceholderPlugin);

    #[cfg(debug_assertions)]
    app.add_plugins(ClientDebugPlugin);

    app.run();
}
