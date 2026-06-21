use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    audio::{AudioCommandsExt, SoundIdentifier},
    game::interact::messages::{DroppedMessage, PickupMessage, UseInHandMessage},
};

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component, Clone)]
#[derive(Default)]
pub struct PlaySoundOnUse {
    pub sound: SoundIdentifier,
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component, Clone)]
pub struct PlaySoundOnPickup {
    pub sound: SoundIdentifier,
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component, Clone)]
pub struct PlaySoundOnDrop {
    pub sound: SoundIdentifier,
}

pub(super) fn play_sound_on_use(
    mut reader: MessageReader<UseInHandMessage>,
    mut commands: Commands,
    query: Query<&PlaySoundOnUse>,
) {
    for event in reader.read() {
        let Ok(play_sound) = query.get(event.target) else {
            continue;
        };

        commands.play_sound(play_sound.sound.clone());
    }
}

pub(super) fn play_sound_on_pickup(
    mut reader: MessageReader<PickupMessage>,
    mut commands: Commands,
    query: Query<&PlaySoundOnPickup>,
) {
    for event in reader.read() {
        let Ok(play_sound) = query.get(event.target) else {
            continue;
        };

        commands.play_sound(play_sound.sound.clone());
    }
}

pub(super) fn play_sound_on_drop(
    mut reader: MessageReader<DroppedMessage>,
    mut commands: Commands,
    query: Query<&PlaySoundOnDrop>,
) {
    for event in reader.read() {
        let Ok(play_sound) = query.get(event.target) else {
            continue;
        };

        commands.play_sound(play_sound.sound.clone());
    }
}
