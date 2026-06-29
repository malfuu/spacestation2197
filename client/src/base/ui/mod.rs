//! Base client ui plugin, windows base code, popups system, map to screen node positioning.
use bevy::prelude::*;

use crate::base::{camera::GameCamera, states::AppState};

pub(super) struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_map_to_screen).run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Default)]
pub enum MapToScreenAnchor {
    #[default]
    Center,
}

#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(Transform)]
pub struct MapToScreen {
    pub anchor: MapToScreenAnchor,
}

fn update_map_to_screen(
    camera_q: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut ui_query: Query<(&GlobalTransform, &mut Node, &ComputedNode, &MapToScreen)>,
) {
    let (camera, camera_transform) = *camera_q;

    for (global_transform, mut node, computed_node, map_to_screen) in ui_query.iter_mut() {
        let world_pos = global_transform.translation();
        let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
            continue;
        };
        let size = computed_node.size();
        let (offset_x, offset_y) = match map_to_screen.anchor {
            MapToScreenAnchor::Center => (size.x / 2.0, size.y / 2.0),
        };
        node.left = px(viewport_pos.x - offset_x);
        node.top = px(viewport_pos.y - offset_y);
    }
}
