pub mod administration;
pub mod customization;
pub mod gamemode;
pub mod ooc;
pub mod round;

use bevy::prelude::*;
use shared::meta::MetaSystems;

use crate::meta::{
    administration::ServerAdministrationPlugin, customization::read_set_customization,
    gamemode::ServerGamemodePlugin, ooc::OocPlugin, round::RoundPlugin,
};

pub(super) struct ServerMetaPlugin;

impl Plugin for ServerMetaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            read_set_customization.in_set(MetaSystems::Inputs),
        )
        .add_plugins(OocPlugin)
        .add_plugins(ServerAdministrationPlugin)
        .add_plugins(RoundPlugin)
        .add_plugins(ServerGamemodePlugin);
    }
}
