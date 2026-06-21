use bevy::prelude::*;

use shared::{
    audio::AudioCommandsExt,
    game::{
        GameplaySystems,
        combat::Weapon,
        interact::{intent::Intent, messages::InteractWithMessage},
    },
};

pub(super) struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AttackMeleeMessage>().add_systems(
            FixedUpdate,
            read_interacts_weapons.in_set(GameplaySystems::Logic),
        );
    }
}

/// Triggered when a mob is attack by another mob
#[derive(Message, Debug, Clone, Copy)]
pub struct AttackMeleeMessage {
    pub user: Entity,
    pub target: Entity,
    pub damage: u32,
    pub weapon: Option<Entity>,
}

fn read_interacts_weapons(
    mut messages: MessageReader<InteractWithMessage>,
    mut commands: Commands,
    weapons: Query<&Weapon>,
) {
    for interaction in messages.read() {
        if !matches!(interaction.intent, Intent::Aggressive) {
            continue;
        }

        let Ok(weapon) = weapons.get(interaction.using) else {
            continue;
        };

        commands.play_sound(weapon.hit_sound.clone());

        commands.write_message(AttackMeleeMessage {
            user: interaction.user,
            target: interaction.target,
            damage: weapon.damage,
            weapon: Some(interaction.using),
        });
    }
}
