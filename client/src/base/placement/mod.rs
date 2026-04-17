//! Not actual placement of entities/tiles
//! But rather visualization of placements.
use bevy::prelude::*;

pub(super) struct ClientPlacementPlugin;

impl Plugin for ClientPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<PlacementGizmos>()
            .add_systems(Update, draw_placement_gizmos);
    }
}

#[derive(Resource, Deref, DerefMut)]
struct CurrentPlacement(());

/// Creates or overrides the previous placement,
/// allowing for a client visualization of placement
fn set_placement(mut commands: Commands) {}

/// Stops drawing any previous placement information
fn end_placement(mut commands: Commands) {
    commands.remove_resource::<CurrentPlacement>();
}

#[derive(GizmoConfigGroup, Reflect, Default)]
struct PlacementGizmos;

fn draw_placement_gizmos(gizmos: Gizmos<PlacementGizmos>) {}
