use bevy::prelude::*;

use bevy_replicon::prelude::*;
use content::prelude::*;

use serde::{Deserialize, Serialize};

pub(super) struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<Ghost>()
            .replicate::<Ghost>()
            .add_client_message::<GhostInput>(Channel::Unreliable);
    }
}

#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct GhostInput;

/// Marks a [`crate::game::mob::Mob`] as a ghost, usually for spectators.
#[derive(Component, Reflect, Clone, Default, Debug, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component, Clone)]
pub struct Ghost;

/// Component for [`crate::game::player::Player`]
#[derive(Component, Reflect, Clone, Debug)]
#[component(immutable)]
#[reflect(Component, Clone)]
pub struct CanSeeGhost;

impl VisibilityFilter for Ghost {
    type ClientComponent = CanSeeGhost;
    type Scope = Entity;

    fn is_visible(&self, client: Entity, component: Option<&Self::ClientComponent>) -> bool {
        info!("visible check: {self:?}, {client:?}, {component:?}");
        component.is_some()
    }
}
