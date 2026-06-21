//! Audio specifications and commands
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

pub(super) struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_server_message::<PlaySoundMessage>(Channel::Unordered);
    }
}

pub type SoundIdentifier = String; // path as identifier

#[derive(Event, Clone, Debug)]
pub struct PlaySound {
    pub sound: SoundIdentifier,
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct PlaySoundMessage {
    pub sound: SoundIdentifier,
}

pub trait AudioCommandsExt {
    fn play_sound(&mut self, sound: impl Into<SoundIdentifier>);
}

impl<'w, 's> AudioCommandsExt for Commands<'w, 's> {
    fn play_sound(&mut self, sound: impl Into<SoundIdentifier>) {
        self.trigger(PlaySound {
            sound: sound.into(),
        });
    }
}
