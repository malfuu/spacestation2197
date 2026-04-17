//! Power related areas, consumers, producers, storages.
use bevy::prelude::*;
use content::prelude::PrototypeComponentAppExt;
use serde::{Deserialize, Serialize};

pub(super) struct PowerPlugin;

impl Plugin for PowerPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<Apc>()
            .prototype_component_no_default::<PowerStorage>()
            .prototype_component_no_default::<PowerProducer>()
            .prototype_component_no_default::<PowerConsumer>();
    }
}

#[derive(Component, Reflect, Clone, Default, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Apc;

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct PowerStorage {
    energy_joules: f32,
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct PowerProducer {
    power_watts: f32,
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct PowerConsumer {
    power_watts: f32,
}
