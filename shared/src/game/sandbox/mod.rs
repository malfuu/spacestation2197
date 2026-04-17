use bevy::{ecs::entity::MapEntities, prelude::*};
use common::PrototypeId;
use serde::{Deserialize, Serialize};

use bevy_replicon::prelude::*;

use crate::game::placement::Placement;

pub(super) struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_mapped_client_message::<SandboxCommands>(Channel::Ordered);
    }
}

/// Marks a player entity as being able to do sandbox commands.
#[derive(Component, Default, Clone, Copy)]
#[component(immutable)]
pub struct Sandboxer;

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
pub enum SandboxCommands {
    Place(Placement),
    SetMixture(Option<PrototypeId>, IVec2), // none if clear at pos
    EraseEntity(Entity),
    EraseTile(IVec2),
}

impl MapEntities for SandboxCommands {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        match self {
            SandboxCommands::EraseEntity(entity) => {
                *entity = entity_mapper.get_mapped(*entity);
            }
            SandboxCommands::EraseTile(_)
            | SandboxCommands::Place(_)
            | SandboxCommands::SetMixture(_, _) => {}
        }
    }
}
