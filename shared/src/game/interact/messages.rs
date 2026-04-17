use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::{hands::Hand, interact::intent::Intent};

/// Sent when a mob interacts another entity
#[derive(Message, Debug, Clone, Copy)]
pub struct InteractMessage {
    pub user: Entity,
    pub target: Entity,
    pub intent: Intent,
}

/// Sent when a handed mob interacts using an item
#[derive(Message, Debug, Clone, Copy)]
pub struct InteractWithMessage {
    pub user: Entity,
    pub using: Entity,
    pub target: Entity,
    pub intent: Intent,
}

/// Sent when a handed mob interacts with an empty hand
#[derive(Message, Debug, Clone, Copy)]
pub struct InteractHandMessage {
    pub user: Entity,
    pub hand: Hand,
    pub target: Entity,
    pub intent: Intent,
}

/// Written when an entity is used in hands.
#[derive(Message, Debug, Clone, Copy)]
pub struct UseInHandMessage {
    pub user: Entity,
    pub target: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct PickupMessage {
    pub user: Entity,
    pub target: Entity,
}

#[derive(Message, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DroppedMessage {
    pub user: Entity,
    pub target: Entity,
}
