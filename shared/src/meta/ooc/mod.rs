use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_replicon::prelude::*;

pub(super) struct OocPlugin;

impl Plugin for OocPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_message::<PlayerOoc>(Channel::Ordered);
    }
}

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
pub struct PlayerOoc(pub String);
