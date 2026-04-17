use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use content::prelude::*;

use bevy_replicon::prelude::*;

pub(super) struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<Item>()
            .prototype_component::<Tool>()
            .prototype_component_with_default(Stack {
                amount: 30,
                max_amount: 30,
            })
            .replicate_once::<Item>();
    }
}

/// Base archetype for free floating items that can be picked up and placed in inventory
#[derive(Component, Default, Reflect, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component)]
pub struct Item;

#[derive(Component, Reflect, Default, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component)]
pub struct Tool;

#[derive(Component, Reflect, Default, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component)]
pub struct Stack {
    pub amount: u32,
    pub max_amount: u32,
}
