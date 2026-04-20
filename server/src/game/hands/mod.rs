use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    game::{
        GameplaySystems,
        containers::Contained,
        hands::{DropInput, Hands, SwitchHandsInput, ThrowInput, UseInput},
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
            (
                read_input_drops,
                read_input_throws,
                read_input_uses,
                read_input_switch_hands,
            )
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

fn read_input_throws(
    mut reader: MessageReader<FromClient<ThrowInput>>,
    mut commands: Commands,
    clients: Query<&Controls, PlayerFilter>,
    mut mobs: Query<(&Transform, Mut<Hands>), AliveMobFilter>,
    mut item_physics: Query<(Option<&Mass>, Forces)>,
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
            continue; // nothing in hands to throw
        };

        let throw_direction = input.direction.normalize_or_zero();

        let throw_spawn_distance = 0.25;
        let throw_spawn_offset =
            vec3(throw_direction.x, 0.0, throw_direction.y) * throw_spawn_distance + Vec3::Y;
        let spawn_position = transform.translation + throw_spawn_offset;

        commands.entity(item_entity).remove::<Contained>().insert((
            Transform::from_translation(spawn_position),
            Position::from(spawn_position),
            RigidBody::Dynamic,
        ));

        if let Ok((mass_opt, mut forces)) = item_physics.get_mut(item_entity) {
            let mass = mass_opt.map(|m| m.0).unwrap_or(1.1); // mass or 1kg
            let throw_energy = 40.0;

            // kinetic energy to momentum
            let momentum = (2.0 * mass * throw_energy).sqrt();
            let throw_impulse = vec3(throw_direction.x, 0.0, throw_direction.y) * momentum;

            forces.apply_linear_impulse(throw_impulse);
        }
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
