//! purely singleton struct for replication
//! as substitute until resources can be replicated.
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

pub struct ManagerPlugin;

impl Plugin for ManagerPlugin {
    fn build(&self, app: &mut App) {
        app.replicate_once::<Manager>();
    }
}

/// Marker for singleton replicated entity,
/// until bevy_replicon implements networked resources.
#[derive(Component, Default, Serialize, Deserialize)]
pub struct Manager;
