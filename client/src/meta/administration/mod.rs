//! Administration functionality for the client.
mod ui;

use bevy::prelude::*;

use crate::meta::administration::ui::ClientAdministrationUiPlugin;

pub(super) struct ClientAdministrationPlugin;

impl Plugin for ClientAdministrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientAdministrationUiPlugin);
    }
}
