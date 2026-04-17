//! Chat text definitions!
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

pub(super) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_server_message::<ChatMessage>(Channel::Ordered);
    }
}

#[derive(Message, Deref, Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage(pub String);
