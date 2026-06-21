//! Audio related controls for the client.

use bevy::prelude::*;
use shared::audio::PlaySoundMessage;

pub(super) struct ClientAudioPlugin;

impl Plugin for ClientAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, read_play_sound);
    }
}

fn read_play_sound(
    mut messages: MessageReader<PlaySoundMessage>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for message in messages.read() {
        commands.spawn((
            AudioPlayer::new(asset_server.load(&message.sound)),
            PlaybackSettings::DESPAWN,
        ));
    }
}
