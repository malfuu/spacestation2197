use bevy::prelude::*;
use serde::Deserialize;

use content::prelude::*;

pub(super) struct MachineryPlugin;

impl Plugin for MachineryPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<Vent>()
            .prototype_component::<Scrubber>();
    }
}

#[derive(Component, Deserialize, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct Vent;

#[derive(Component, Deserialize, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct Scrubber;
