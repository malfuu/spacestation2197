//! Everything not related to the gameplay loop
//! happens outside game logic e.g. ooc, external services, so on
pub mod administration;
pub mod customization;
pub mod gamemode;
pub mod manager;
pub mod ooc;
pub mod player;
pub mod round;

use bevy::prelude::*;

use crate::{
    game::GamePlugin,
    meta::{
        administration::AdministrationPlugin, customization::CustomizationPlugin,
        gamemode::GamemodePlugin, manager::ManagerPlugin, ooc::OocPlugin, player::PlayerPlugin,
        round::RoundPlugin,
    },
};

pub(super) struct MetaPlugin;

impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<GamePlugin>());

        app.configure_sets(
            FixedUpdate,
            (MetaSystems::Inputs, MetaSystems::Logic, MetaSystems::Final).chain(),
        )
        .add_plugins(ManagerPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(AdministrationPlugin)
        .add_plugins(RoundPlugin)
        .add_plugins(CustomizationPlugin)
        .add_plugins(OocPlugin)
        .add_plugins(GamemodePlugin);
    }
}

/// Runs all meta related stuff e.g. Round management, OOC, Player management, etc...
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetaSystems {
    /// Receives player inputs
    Inputs,
    /// Normal systems here
    Logic,
    /// Finalizing and sending messages to the client
    Final,
}
