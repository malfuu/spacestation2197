use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker for active chunks in simulation.
/// Thought it is unutilized right now
#[derive(Component, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component, Default, Debug, Serialize, Deserialize)]
#[component(storage = "SparseSet")]
pub struct Excited;
