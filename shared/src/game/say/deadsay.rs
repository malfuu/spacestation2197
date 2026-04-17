use bevy::prelude::*;

/// Something an entity says to deadchat
#[derive(Event, Debug)]
pub struct EntityDeadSay {
    pub speaker: Entity,
    pub message: String,
}
