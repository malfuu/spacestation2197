use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::AtmosTick;

/// Marker for active chunks in simulation.
/// A chunk that is excited will:
/// - perform exchanges in-chunk and to neighboring chunks.
/// - clear gas mixtures exposed to space
/// - perform reactions (todo)
/// - expand hotspots (todo)
#[derive(Component, Default, Debug, Serialize, Deserialize)]
#[component(storage = "SparseSet")]
pub struct Excited {
    pub last_active_tick: AtmosTick,
}

#[derive(Component)]
pub struct ProcessedTick(pub u32);

impl Default for ProcessedTick {
    fn default() -> Self {
        Self(u32::MAX)
    }
}
