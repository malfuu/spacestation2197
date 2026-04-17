//! Universal Client Application states, including useful run conditions.
//! All substates should depend on this.

use bevy::prelude::*;

pub(super) struct ClientStatesPlugin;

impl Plugin for ClientStatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}

#[derive(States, PartialEq, Eq, Hash, Default, Debug, Clone, Copy)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    Editor,
}
