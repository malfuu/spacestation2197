mod sounds;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use content::prelude::*;

use crate::game::{
    hands::sounds::{
        PlaySoundOnDrop, PlaySoundOnPickup, PlaySoundOnUse, play_sound_on_drop,
        play_sound_on_pickup, play_sound_on_use,
    },
    interact::messages::{DroppedMessage, PickupMessage, UseInHandMessage},
};

pub(super) struct HandsPlugin;

impl Plugin for HandsPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<Hands>()
            .prototype_component_no_default::<PlaySoundOnUse>()
            .prototype_component_no_default::<PlaySoundOnPickup>()
            .prototype_component_no_default::<PlaySoundOnDrop>()
            .replicate::<Hands>()
            .replicate::<PlaySoundOnUse>()
            .replicate::<PlaySoundOnPickup>()
            .replicate::<PlaySoundOnDrop>()
            .add_client_message::<DropInput>(Channel::Unreliable)
            .add_client_message::<UseInput>(Channel::Unreliable)
            .add_client_message::<ThrowInput>(Channel::Unreliable)
            .add_client_message::<SwitchHandsInput>(Channel::Unreliable)
            .add_message::<UseInHandMessage>()
            .add_message::<PickupMessage>()
            .add_message::<DroppedMessage>()
            .add_systems(
                Update,
                (play_sound_on_use, play_sound_on_pickup, play_sound_on_drop),
            );
    }
}

#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct DropInput;

#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ThrowInput {
    pub direction: Vec2,
}

#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct UseInput;

#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SwitchHandsInput; // perhaps active hand should be client side?

/// Keep in mind that in the future there might be more hands than left and right.
#[derive(Reflect, Clone, Copy, Debug, Hash, Default, Serialize, Deserialize)]
pub enum Hand {
    #[default]
    Left,
    Right,
}

/// Entities that are held go hand in hand with [`crate::game::containers::Contained`].
#[derive(Component, Reflect, Clone, Debug, Default, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct Hands {
    pub active: Hand,
    #[entities]
    left: Option<Entity>,
    #[entities]
    right: Option<Entity>,
}

impl Hands {
    pub fn switch(&mut self) {
        self.active = match self.active {
            Hand::Left => Hand::Right,
            Hand::Right => Hand::Left,
        }
    }

    pub fn get_hand(&self) -> Option<Entity> {
        self.get(self.active)
    }

    pub fn get_active(&self) -> Option<Entity> {
        self.get(self.active)
    }

    pub fn get_active_mut(&mut self) -> &mut Option<Entity> {
        self.get_mut(self.active)
    }

    pub fn get(&self, hand: Hand) -> Option<Entity> {
        match hand {
            Hand::Left => self.left,
            Hand::Right => self.right,
        }
    }

    pub fn get_mut(&mut self, hand: Hand) -> &mut Option<Entity> {
        match hand {
            Hand::Left => &mut self.left,
            Hand::Right => &mut self.right,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Option<Entity>> {
        [&self.left, &self.right].into_iter()
    }
}
