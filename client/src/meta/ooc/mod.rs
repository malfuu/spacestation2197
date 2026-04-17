//! (simple) OOC channel functionality.

use bevy::prelude::*;
use shared::meta::ooc::PlayerOoc;

use crate::placeholder::chat::OocEnter;

pub(super) struct ClientOocPlugin;

impl Plugin for ClientOocPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ooc_enter);
    }
}

fn on_ooc_enter(say: On<OocEnter>, mut commands: Commands) {
    commands.write_message(PlayerOoc(say.0.clone()));
}
