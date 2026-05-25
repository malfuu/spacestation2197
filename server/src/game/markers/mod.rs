use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::game::markers::Marker;

pub(super) struct MarkersPlugin;

impl Plugin for MarkersPlugin {
    fn build(&self, app: &mut App) {
        app.add_visibility_filter::<Marker>();
    }
}
