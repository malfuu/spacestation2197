//! Audio specifications and commands
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

pub(super) struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_server_message::<PlaySoundMessage>(Channel::Unordered)
            .add_message::<PlaySoundGlobally>()
            .add_message::<PlaySoundLocally>();
    }
}

pub type SoundIdentifier = String; // path as identifier

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub enum PlaySoundMessage {
    Global {
        sound: SoundIdentifier,
    },
    Spatial {
        sound: SoundIdentifier,
        position: Vec3,
        range: f32,
    },
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct PlaySoundGlobally {
    pub sound: SoundIdentifier,
}

impl From<PlaySoundGlobally> for PlaySoundMessage {
    fn from(m: PlaySoundGlobally) -> Self {
        Self::Global { sound: m.sound }
    }
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct PlaySoundLocally {
    pub sound: SoundIdentifier,
    pub position: Vec3,
    pub range: f32,
}

impl From<PlaySoundLocally> for PlaySoundMessage {
    fn from(m: PlaySoundLocally) -> Self {
        Self::Spatial {
            sound: m.sound,
            position: m.position,
            range: m.range,
        }
    }
}

pub trait AudioCommandsExt {
    fn play_sound_globally(&mut self, sound: impl Into<SoundIdentifier>);
    fn play_sound_locally(&mut self, sound: impl Into<SoundIdentifier>, position: Vec3);
}

impl<'w, 's> AudioCommandsExt for Commands<'w, 's> {
    fn play_sound_globally(&mut self, sound: impl Into<SoundIdentifier>) {
        self.write_message(PlaySoundGlobally {
            sound: sound.into(),
        });
    }

    fn play_sound_locally(&mut self, sound: impl Into<SoundIdentifier>, position: Vec3) {
        self.write_message(PlaySoundLocally {
            sound: sound.into(),
            position,
            range: 16.0,
        });
    }
}
