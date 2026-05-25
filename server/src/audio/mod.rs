use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    audio::{PlaySoundGlobally, PlaySoundLocally, PlaySoundMessage},
    meta::MetaSystems,
};

pub(super) struct ServerAudioPlugin;

impl Plugin for ServerAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, transmit_sounds.in_set(MetaSystems::Final));
    }
}

fn transmit_sounds(
    mut commands: Commands,
    mut global_sounds: MessageReader<PlaySoundGlobally>,
    mut local_sounds: MessageReader<PlaySoundLocally>,
) {
    for message in global_sounds.read() {
        commands.write_message(ToClients::<PlaySoundMessage> {
            targets: SendTargets::All,
            message: message.clone().into(),
        });
    }

    for message in local_sounds.read() {
        commands.write_message(ToClients::<PlaySoundMessage> {
            targets: SendTargets::All,
            message: message.clone().into(),
        });
    }
}
