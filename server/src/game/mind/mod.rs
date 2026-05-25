use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    game::{
        GameplaySystems,
        mind::{OwnMobMessage, TakeOverMessage},
        mob::Mob,
        player::Player,
    },
    utils::filters::MobFilter,
};

use crate::utils::{SpawnMethod, SpawnerCommandsExt};

pub(super) struct MindPlugin;

impl Plugin for MindPlugin {
    fn build(&self, app: &mut App) {
        app.sync_related_entities::<Controlled>()
            .add_systems(
                FixedUpdate,
                on_read_takeovers.in_set(GameplaySystems::Final),
            )
            .add_observer(on_insert_controlled)
            .add_observer(on_remove_controlled);
    }
}

/// Added to [`shared::game::mob::Mob`]
#[derive(Component, Deref, Debug)]
#[component(immutable)]
#[relationship(relationship_target = Controls)]
pub struct Controlled(pub Entity);

/// What mob the player controls.
#[derive(Component, Deref)]
#[relationship_target(relationship = Controlled)]
pub struct Controls(Entity);

pub(crate) fn on_insert_controlled(
    add: On<Insert, Controlled>,
    mut commands: Commands,
    mobs: Query<&Controlled, MobFilter>,
) {
    let controlled = mobs.get(add.entity).expect("Controlled should exist.");
    let player_entity = controlled.0;

    commands.write_message(TakeOverMessage {
        client_entity: player_entity,
        target_mob: add.entity,
    });
}

pub(crate) fn on_remove_controlled(
    trigger: On<Despawn, Controlled>,
    mut commands: Commands,
    query: Query<(&Controlled, &Transform), MobFilter>,
) {
    let Ok((controlled, transform)) = query.get(trigger.entity) else {
        return;
    };

    let player_entity = controlled.0;
    let ghost_pos = transform.translation;

    commands.spawn_player(
        player_entity,
        "ghost".to_string(),
        SpawnMethod::Position(ghost_pos.xz()),
    )
}

pub(crate) fn on_read_takeovers(
    mut messages: MessageReader<TakeOverMessage>,
    mut commands: Commands,
    _players: Query<&Player>,
    _mobs: Query<&Mob>,
) {
    for takeover in messages.read() {
        commands.write_message(ToClients {
            targets: SendTargets::Single(ClientId::Client(takeover.client_entity)),
            message: OwnMobMessage(Some(takeover.target_mob)),
        });
    }
}
