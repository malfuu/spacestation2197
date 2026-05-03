//! Various UI pieces that are programmable
//! You might find this module empty as currently a lot of UI is scattered egui functions
mod following_text;

use bevy::prelude::*;

use crate::base::ui::following_text::FollowingTextPlugin;

pub(super) struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FollowingTextPlugin);
    }
}
