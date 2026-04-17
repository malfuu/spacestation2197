use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_replicon::prelude::*;

use content::prelude::*;

pub(super) struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component_with_default(Light {
            color: Srgba::WHITE,
            intensity: 10_000.0,
            range: 12.0,
        })
        .replicate::<Light>();
    }
}

/// Networkable point light
#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component)]
#[component(immutable)]
pub struct Light {
    pub color: Srgba,
    pub intensity: f32, // lumens
    pub range: f32,
}
