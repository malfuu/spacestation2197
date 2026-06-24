//! Handling of entities to be ready for client side use.
use bevy::{prelude::*, world_serialization::WorldInstanceReady};
use bevy_replicon::prelude::*;

use common::EntityTag;
use content::prelude::*;

use crate::base::states::AppState;

pub(super) struct ClientEntityPlugin;

impl Plugin for ClientEntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_remote)
            .add_observer(on_add_entity_tag)
            .add_observer(apply_pickable);
    }
}

fn on_add_remote(add: On<Add, Remote>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .insert(DespawnOnExit(ClientState::Connected));
}

fn on_add_entity_tag(
    add: On<Add, EntityTag>,
    mut commands: Commands,
    entity_tags: Query<&EntityTag>,
    asset_server: Res<AssetServer>,
    registry: Res<Prototypes>,
    state: Res<State<AppState>>,
) {
    let entity_tag = entity_tags.get(add.entity).expect("EntityTag must exist.");

    let mesh = registry
        .get::<EntityPrototype>(PROTOTYPE_CATEGORY_ENTITY, &entity_tag.0)
        .expect("there should be an entity for this!")
        .mesh
        .clone();

    commands.entity(add.entity).insert((WorldAssetRoot(
        asset_server.load(format!("{}#Scene0", mesh)),
    ),));
}

pub fn apply_pickable(
    on: On<WorldInstanceReady>,
    mut commands: Commands,
    query: Query<(Entity, Option<&EntityTag>)>,
    children: Query<&Children>,
) {
    let Ok((parent, entity)) = query.get(on.entity) else {
        return;
    };

    let pickable = if entity.is_some() {
        Pickable::default()
    } else {
        return;
    };

    for child in children.iter_descendants(parent) {
        commands.entity(child).insert(pickable);
    }
}
