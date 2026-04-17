use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_replicon::prelude::*;

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.replicate_once::<Player>();
    }
}

/// Marker for player entities
#[derive(Component, Default, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub struct Player;
