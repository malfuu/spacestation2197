use bevy::prelude::*;
use bevy_replicon::prelude::*;

use common::EntityTag;
use shared::{
    audio::AudioCommandsExt,
    defines::MOB_REACH,
    game::{
        GameplaySystems,
        containers::Contained,
        hands::Hands,
        interact::{
            InteractCooldown, InteractInput,
            intent::Intent,
            messages::{InteractHandMessage, InteractMessage, InteractWithMessage, PickupMessage},
        },
        items::Item,
        mob::{
            Mob,
            health::{Dead, Health},
        },
    },
    utils::filters::{MobFilter, PlayerFilter},
};

use crate::{
    game::{combat::AttackMeleeMessage, mind::Controls},
    utils::MessageCommandsExt,
};

pub(super) struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<InteractMessage>()
            .add_message::<InteractWithMessage>()
            .add_message::<InteractHandMessage>()
            .add_systems(
                FixedUpdate,
                (read_input_interacts,).in_set(GameplaySystems::Inputs),
            )
            .add_systems(
                FixedUpdate,
                (
                    read_interacts,
                    read_interact_with_item,
                    read_interact_empty_hand,
                    read_attacks,
                )
                    .in_set(GameplaySystems::Logic),
            )
            .add_observer(on_hug);
    }
}

fn read_input_interacts(
    mut reader: MessageReader<FromClient<InteractInput>>,
    mut commands: Commands,
    clients: Query<&Controls, PlayerFilter>,
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

        let target = input.target;
        commands.write_message(InteractMessage {
            user: mob_entity,
            target,
            intent: input.intent,
        });
    }
}

#[derive(Event)]
pub struct Hug {
    user: Entity,
    target: Entity,
}

type InteractMobsQueryData<'a> = (
    Option<&'a Dead>,
    Option<&'a Hands>,
    Option<&'a mut InteractCooldown>,
);

fn read_interacts(
    mut reader: MessageReader<InteractMessage>,
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut mobs: Query<InteractMobsQueryData, MobFilter>,
    transforms: Query<&Transform>,
) {
    let now = time.elapsed();
    for interaction in reader.read() {
        let Ok((dead_opt, hands_opt, cooldown_opt)) = mobs.get_mut(interaction.user) else {
            continue;
        };

        if dead_opt.is_some() {
            continue;
        }

        if let Some(cooldown) = &cooldown_opt
            && now < cooldown.ready_at
        {
            // Still on cooldown
            continue;
        }

        let Some(hands) = hands_opt else {
            // handless mobs dont interact for now :(
            continue;
        };

        let origin = transforms
            .get(interaction.user)
            .expect("Should have transform.");
        let target = transforms
            .get(interaction.target)
            .expect("Should have transform.");

        let a = origin.translation.xz();
        let b = target.translation.xz();
        let dist = a.distance(b);
        if dist > MOB_REACH {
            continue;
        }

        if let Some(mut cooldown) = cooldown_opt {
            cooldown.ready_at = now + cooldown.duration;
        }

        match hands.get_hand() {
            Some(entity_in_hand) => {
                commands.write_message(InteractWithMessage {
                    user: interaction.user,
                    using: entity_in_hand,
                    target: interaction.target,
                    intent: interaction.intent,
                });
            }
            None => {
                commands.write_message(InteractHandMessage {
                    user: interaction.user,
                    hand: hands.active,
                    target: interaction.target,
                    intent: interaction.intent,
                });
            }
        }
    }
}

fn read_interact_with_item(mut reader: MessageReader<InteractWithMessage>) {
    for interaction in reader.read() {
        if interaction.using == interaction.target {
            // i wonder when this footgun will explode later
            warn!("Entity {:?} used on itself!", interaction.using);
        }
    }
}

enum TargetKind {
    Mob,
    Item,
    None,
}

type InteractEmptyHandQueryData<'a> = (Option<&'a Mob>, Option<&'a Item>, Option<Mut<'a, Hands>>);

fn read_interact_empty_hand(
    mut reader: MessageReader<InteractHandMessage>,
    mut commands: Commands,
    // if we are doing a full optional query, why dont we just use the world?
    // god this is going this whole project will be so spaghetti
    mut entities: Query<InteractEmptyHandQueryData>,
    transforms: Query<&GlobalTransform>,
) {
    for interaction in reader.read() {
        let kind = {
            let (mob, item, _) = entities
                .get(interaction.target)
                .expect("optionless query fail");

            if mob.is_some() {
                TargetKind::Mob
            } else if item.is_some() {
                TargetKind::Item
            } else {
                TargetKind::None
            }
        };

        match kind {
            TargetKind::Mob => {
                on_interact_mob(&mut commands, *interaction, &transforms);
            }
            TargetKind::Item => {
                on_interact_item(&mut commands, *interaction, &mut entities);
            }
            TargetKind::None => {}
        }
    }
}

fn on_interact_mob(
    commands: &mut Commands,
    interaction: InteractHandMessage,
    transforms: &Query<&GlobalTransform>,
) {
    match interaction.intent {
        Intent::Passive => {
            commands.trigger(Hug {
                user: interaction.user,
                target: interaction.target,
            });
        }
        Intent::Aggressive => {
            let puncher = transforms
                .get(interaction.user)
                .expect("Puncher should have transform.");

            commands.play_sound_locally("sounds/punch1.ogg", puncher.translation());

            commands.write_message(AttackMeleeMessage {
                user: interaction.user,
                target: interaction.target,
                damage: 34,
                weapon: None,
            });
        }
    };
}

fn on_hug(
    hug: On<Hug>,
    mut commands: Commands,
    names: Query<&Name>,
    transforms: Query<&Transform>,
) {
    let from_name = names.get(hug.user).expect("from must have name");
    let to_name = names.get(hug.target).expect("to must have name");
    commands.broadcast_chat_message(format!("{} hugged {}.", from_name, to_name));

    let hugged_position = transforms
        .get(hug.target)
        .expect("hugged should have transform.");

    commands.play_sound_locally("sounds/hug.ogg", hugged_position.translation);
}

fn on_interact_item(
    commands: &mut Commands,
    interaction: InteractHandMessage,
    entities: &mut Query<InteractEmptyHandQueryData>,
) {
    let (_, _, hands_opt) = entities
        .get_mut(interaction.user)
        .expect("optionless query fail");

    let Some(mut hands) = hands_opt else {
        warn!("InteractHand from handless mob!");
        return;
    };

    let hand = hands.get_mut(interaction.hand);
    *hand = Some(interaction.target);

    commands
        .entity(interaction.target)
        .insert(Contained(interaction.user));

    commands.write_message(PickupMessage {
        user: interaction.user,
        target: interaction.target,
    });
}

fn read_attacks(
    mut reader: MessageReader<AttackMeleeMessage>,
    mut commands: Commands,
    mut mobs: Query<&mut Health, MobFilter>,
    names: Query<&Name>,
    tags: Query<&EntityTag>,
) {
    for AttackMeleeMessage {
        user,
        target,
        damage,
        weapon,
    } in reader.read()
    {
        let Ok(mut health) = mobs.get_mut(*target) else {
            continue;
        };

        info!("User {user:?} has attacked {target:?} by {damage:?}");

        let user_name = names.get(*user).unwrap();
        let target_name = names.get(*target).unwrap();

        let message = match weapon {
            Some(weapon) => {
                let weapon_name = &tags.get(*weapon).expect("weapon should have tag").0;

                format!("{user_name} attacked {target_name} with a {weapon_name}.")
            }
            None => format!("{user_name} punched {target_name}!"),
        };

        commands.broadcast_chat_message(message);

        health.amount -= *damage as i32;
    }
}
