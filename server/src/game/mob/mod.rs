use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    game::{
        GameplaySystems,
        mob::{MoveInput, controller::AccumulatedInput, health::Dead},
    },
    utils::filters::{MobFilter, PlayerFilter},
};

use crate::{
    game::{
        mind::Controls,
        mob::health::{die, on_death},
    },
    networking::ServerClientEntity,
};

pub mod health;

pub(super) struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (read_input_moves,).in_set(GameplaySystems::Inputs),
        )
        .add_systems(FixedUpdate, (die,).chain().in_set(GameplaySystems::Logic))
        .add_observer(on_death);
    }
}

pub(super) fn read_input_moves(
    mut reader: MessageReader<FromClient<MoveInput>>,
    _server_client: Res<ServerClientEntity>,
    clients: Query<&Controls, PlayerFilter>,
    mut mobs: Query<&mut AccumulatedInput, (MobFilter, Without<Dead>)>,
) {
    for input in reader.read() {
        let ClientId::Client(client_entity) = input.client_id else {
            continue;
        };

        let Ok(owner) = clients.get(client_entity) else {
            continue;
        };

        let Some(mob_entity) = owner.iter().next() else {
            warn!("received input for no associated mob");
            continue;
        };

        let Ok(mut accum_input) = mobs.get_mut(mob_entity) else {
            continue;
        };

        let direction = input.message.0;
        accum_input.last = Some(direction);
    }
}
