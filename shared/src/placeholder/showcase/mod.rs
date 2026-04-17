//! Temporary module for first playtest game
//! Will contain hardcoded components and miscellaneous things
use bevy::prelude::*;
use serde::Deserialize;

use content::prelude::*;

pub(crate) struct ShowcasePlugin;

impl Plugin for ShowcasePlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<TaskSpawner>()
            .prototype_component::<SimpleTask>();
    }
}

#[derive(Component, Deserialize, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct TaskSpawner;

/// A task to be done
#[derive(Component, Deserialize, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct SimpleTask; // named as such as to avoid colliding w/ bevy's Task
