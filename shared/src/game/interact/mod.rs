//! Entity to Entity interactions, includes definitions and common cases
//! for interactions.
pub mod intent;
pub mod messages;

use std::time::Duration;

use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use content::prelude::*;

use crate::game::interact::intent::Intent;

pub(super) struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<InteractCooldown>()
            .prototype_component::<Interactable>()
            .replicate_once::<Interactable>()
            .add_mapped_client_message::<InteractInput>(Channel::Unreliable);
    }
}

#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct InteractInput {
    pub target: Entity,
    pub intent: Intent,
}

impl MapEntities for InteractInput {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.target = entity_mapper.get_mapped(self.target);
    }
}

/// For now, it only creates outline on client
#[derive(Component, Reflect, Clone, Default, Debug, Serialize, Deserialize)]
#[reflect(Component, Clone, Default)]
pub struct Interactable;

/// Sets an interact cooldown for a mob.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct InteractCooldown {
    pub ready_at: Duration,
    // How long the cooldown lasts
    pub duration: Duration,
}

impl Default for InteractCooldown {
    fn default() -> Self {
        Self {
            ready_at: Duration::ZERO,
            duration: Duration::from_millis(500),
        }
    }
}
