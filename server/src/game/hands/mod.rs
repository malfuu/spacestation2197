use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    game::{
        GameplaySystems,
        containers::Contained,
        hands::{DropInput, Hands, SwitchHandsInput, UseInput},
        interact::messages::{DroppedMessage, UseInHandMessage},
        mob::health::Dead,
    },
    utils::filters::{MobFilter, PlayerFilter},
};

use crate::game::mind::Controls;

pub(super) struct HandsPlugin;

impl Plugin for HandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (read_input_drops, read_input_uses, read_input_switch_hands)
                .in_set(GameplaySystems::Inputs),
        );
    }
}

type AliveMobFilter = (MobFilter, Without<Dead>);

fn read_input_drops(
    mut reader: MessageReader<FromClient<DropInput>>,
    mut commands: Commands,
    clients: Query<&Controls, PlayerFilter>,
    mut mobs: Query<(&Transform, Mut<Hands>), AliveMobFilter>,
) {
    for input in reader.read() {
        let ClientId::Client(client_entity) = input.client_id else {
            continue;
        };

        let Ok(owner) = clients.get(client_entity) else {
            continue;
        };

        let Some(mob_entity) = owner.iter().next() else {
            continue;
        };

        let Ok((transform, mut hands)) = mobs.get_mut(mob_entity) else {
            continue;
        };

        let Some(item_entity) = std::mem::take(hands.get_active_mut()) else {
            continue; // nothing in hands
        };

        let drop_off_position = transform.translation + transform.rotation * Vec3::new(0., 1., 1.);

        commands.entity(item_entity).remove::<Contained>().insert((
            Transform::from_translation(drop_off_position),
            Position::from(drop_off_position),
        ));

        commands.write_message(DroppedMessage {
            user: mob_entity,
            target: item_entity,
        });
    }
}

fn read_input_uses(
    mut reader: MessageReader<FromClient<UseInput>>,
    mut commands: Commands,
    clients: Query<&Controls, PlayerFilter>,
    mobs: Query<&Hands, AliveMobFilter>,
) {
    for input in reader.read() {
        let ClientId::Client(client_entity) = input.client_id else {
            continue;
        };

        let Ok(owner) = clients.get(client_entity) else {
            continue;
        };

        let Some(mob_entity) = owner.iter().next() else {
            continue;
        };

        let Ok(hands) = mobs.get(mob_entity) else {
            continue;
        };

        let Some(item_entity) = hands.get_active() else {
            continue;
        };

        commands.write_message(UseInHandMessage {
            user: mob_entity,
            target: item_entity,
        });
    }
}

fn read_input_switch_hands(
    mut reader: MessageReader<FromClient<SwitchHandsInput>>,
    clients: Query<&Controls, PlayerFilter>,
    mut mobs: Query<&mut Hands, AliveMobFilter>,
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

        let Ok(mut hands) = mobs.get_mut(mob_entity) else {
            continue;
        };

        hands.switch();
    }
}
