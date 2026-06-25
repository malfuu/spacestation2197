use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::audio::{PlaySound, PlaySoundMessage};

use crate::is_authority;

pub(super) struct ServerAudioPlugin;

impl Plugin for ServerAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_play_sound.run_if(is_authority));
    }
}

fn on_play_sound(play_sound: On<PlaySound>, mut commands: Commands) {
    commands.write_message(ToClients::<PlaySoundMessage> {
        targets: SendTargets::All,
        message: PlaySoundMessage {
            sound: play_sound.sound.clone(),
        },
    });
}
