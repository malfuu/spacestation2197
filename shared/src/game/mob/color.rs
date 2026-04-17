use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// skin color that is on the rainbow spectrum!!! for showcase
#[derive(Component, Serialize, Deserialize)]
pub struct SkinColor(pub Srgba);
