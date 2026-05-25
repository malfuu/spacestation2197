use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use content::prelude::*;

pub(super) struct MarkerPlugin;

impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<Marker>();
    }
}

/// Editor-only markers that have invisible effects on gameplay (e.g. spawners)
#[derive(Component, Default, Reflect, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component)]
pub struct Marker;

impl VisibilityFilter for Marker {
    type ClientComponent = Self;
    type Scope = Entity;

    fn is_visible(&self, _client: Entity, _component: Option<&Self::ClientComponent>) -> bool {
        false // shouldnt really be seen in game
    }
}
