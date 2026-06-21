//! Very rudimentary and hardcoded gamemode implementation

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

pub struct GamemodePlugin;

impl Plugin for GamemodePlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<Gamemode>();
    }
}

/// Singleton gamemode for [`crate::meta::manager::Manager`].
#[derive(Component, Reflect, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Component, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Gamemode {
    #[default]
    Extended,
    Sandbox,
}

impl std::fmt::Display for Gamemode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Gamemode::Extended => "Extended",
            Gamemode::Sandbox => "Sandbox",
        };

        write!(f, "{text}")
    }
}
