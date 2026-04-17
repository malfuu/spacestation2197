use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use content::prelude::*;

use crate::audio::SoundIdentifier;

pub(super) struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component_no_default::<Weapon>();
    }
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component, Clone)]
pub struct Weapon {
    pub damage: u32,
    pub hit_sound: SoundIdentifier,
}
