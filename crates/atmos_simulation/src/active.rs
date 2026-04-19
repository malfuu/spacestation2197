use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::AtmosTick;

/// Marker for active chunks in simulation.
/// A chunk that is active will:
/// - perform exchanges internally and on neighboring chunks.
/// - perform reactions
/// - update hotspots
/// - space tiles
#[derive(Component, Default, Debug, Serialize, Deserialize)]
#[component(storage = "SparseSet")]
pub struct Active {
    pub last_active_tick: AtmosTick,
}

#[derive(Component)]
pub(crate) struct ProcessedTick(pub u32);

impl Default for ProcessedTick {
    fn default() -> Self {
        Self(u32::MAX)
    }
}
