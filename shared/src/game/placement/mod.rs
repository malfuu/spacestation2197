//! Placement of entities in the world, w/ possible rules.
//! Really, how entities are placed in the world should be done [`Placement`],
use bevy::prelude::*;
use common::{PrototypeId, TileTag};
use serde::{Deserialize, Serialize};

pub(super) struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Placement {
    Entity {
        entity: PrototypeId,
        position: Vec2,
    },
    Tile {
        tile: TileTag,
        start: IVec2,
        end: IVec2,
    },
}
