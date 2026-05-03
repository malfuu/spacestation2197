//! Various client menu UI implementations in the game.
pub mod escape_menu;
pub mod main_menu;

use bevy::prelude::*;

use crate::base::menus::{escape_menu::ClientEscapeMenuPlugin, main_menu::ClientMainMenuPlugin};

pub(super) struct ClientMenusPlugin;

impl Plugin for ClientMenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientMainMenuPlugin)
            .add_plugins(ClientEscapeMenuPlugin);
    }
}
