pub mod color;
pub mod controller;
pub mod health;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_replicon::prelude::*;
use content::prelude::*;

use crate::game::{
    GameplaySystems,
    mob::{
        color::SkinColor,
        controller::{
            MobController, clear_accumulated_input, controller_move_and_slide,
            controller_update_rotation, controller_update_velocity,
        },
        health::{Dead, Health},
    },
};

pub(super) struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<Mob>()
            .prototype_component_with_default(Health { amount: 100 })
            .prototype_component::<MobController>()
            .replicate_once::<Mob>()
            .replicate::<Health>()
            .replicate::<Dead>()
            .replicate::<SkinColor>()
            .add_client_message::<MoveInput>(Channel::Unreliable)
            .add_systems(
                FixedUpdate,
                (
                    controller_update_velocity,
                    controller_update_rotation,
                    controller_move_and_slide,
                    clear_accumulated_input,
                )
                    .chain()
                    .in_set(GameplaySystems::Logic),
            );
    }
}

#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct MoveInput(pub Vec2);

/// Base archetype for an entity that is capable of producing movement,
/// and that can be controlled by players or NPCs.
#[derive(Component, Reflect, Clone, Default, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component, Clone)]
pub struct Mob;
