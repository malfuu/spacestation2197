use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

pub(super) struct ContainersPlugin;

impl Plugin for ContainersPlugin {
    fn build(&self, app: &mut App) {
        app.sync_related_entities::<Contained>()
            .replicate::<Contained>();
    }
}

/// Entity is inside other entity - not necessarily in their inventory or hand
/// Hooked to [`ChildOf`] to acquire relative [`GlobalTransform`].
#[derive(Component, Reflect, Deref, Serialize, Deserialize)]
#[component(storage = "SparseSet")]
#[relationship(relationship_target = Contains)]
#[reflect(Component)]
pub struct Contained(#[entities] pub Entity);

/// Entities that this entity has inside.
#[derive(Component, Reflect, Deref)]
#[relationship_target(relationship = Contained, linked_spawn)]
#[reflect(Component)]
pub struct Contains(Vec<Entity>);
