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
    transforms: Query<&Transform>,
) {
    for interaction in messages.read() {
        if !matches!(interaction.intent, Intent::Aggressive) {
            continue;
        }

        let Ok(weapon) = weapons.get(interaction.using) else {
            continue;
        };

        let transform = transforms
            .get(interaction.user)
            .expect("Attacker should have transform.");

        commands.play_sound_locally(weapon.hit_sound.clone(), transform.translation);

        commands.write_message(AttackMeleeMessage {
            user: interaction.user,
            target: interaction.target,
            damage: weapon.damage,
            weapon: Some(interaction.using),
        });
    }
}
