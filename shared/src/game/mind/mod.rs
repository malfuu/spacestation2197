//! All [`crate::game::player::Player`] to [`crate::game::mob::Mob`] Mind related.
use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

use bevy_replicon::prelude::*;

pub(super) struct MindPlugin;

impl Plugin for MindPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TakeOverMessage>()
            .add_mapped_server_message::<OwnMobMessage>(Channel::Ordered);
    }
}

/// Deferred Message to delay [`OwnMobMessage`].
#[derive(Message, Debug)]
pub struct TakeOverMessage {
    pub client_entity: Entity,
    pub target_mob: Entity,
}

impl TakeOverMessage {
    pub fn new(client_entity: Entity, target_mob: Entity) -> Self {
        Self {
            client_entity,
            target_mob,
        }
    }
}

/// Server to client message of what entity the player now controls (or does not)
#[derive(Message, Deref, Serialize, Deserialize, Clone, Copy)]
pub struct OwnMobMessage(pub Option<Entity>);

impl MapEntities for OwnMobMessage {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = self.0.map(|x| entity_mapper.get_mapped(x));
    }
}
