use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct Dead; // https://www.youtube.com/watch?v=Z7vHGnIYbM8

/// Triggered when a mob dies
#[derive(Event, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Death {
    pub target: Entity,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Deref, DerefMut)]
#[reflect(Component)]
pub struct Health {
    pub amount: i32,
}

impl Health {
    pub fn is_in_critical(&self) -> bool {
        self.amount <= 0
    }

    pub fn is_lethal(&self) -> bool {
        self.amount <= -100
    }
}
