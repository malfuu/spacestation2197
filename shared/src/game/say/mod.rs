pub mod deadsay;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use content::prelude::*;

pub(super) struct SayPlugin;

impl Plugin for SayPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<Speaker>()
            .prototype_component::<Listener>()
            .add_client_message::<SayInput>(Channel::Unordered);
    }
}

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
pub struct SayInput(pub String);

/// Marks a Entity as being able to speak
#[derive(Component, Reflect, Default, Serialize, Deserialize, Clone)]
#[reflect(Component)]
pub struct Speaker;

/// Marks a Entity as being able to listen
#[derive(Component, Reflect, Default, Serialize, Deserialize, Clone)]
#[reflect(Component)]
pub struct Listener;

/// Something an entity says.
#[derive(Event, Debug)]
pub struct EntitySay {
    pub speaker: Entity,
    pub message: String,
}

/// Something an entity has heart from another entity.
#[derive(Event, Debug)]
pub struct EntityListen {
    pub speaker: Entity,
    pub listener: Entity,
    pub message: String,
}
