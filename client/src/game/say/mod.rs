use bevy::prelude::*;
use shared::game::say::SayInput;

use crate::placeholder::chat::SayEnter;

pub(super) struct ClientSayPlugin;

impl Plugin for ClientSayPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_say_enter);
    }
}

fn on_say_enter(say: On<SayEnter>, mut commands: Commands) {
    commands.write_message(SayInput(say.0.clone()));
}
