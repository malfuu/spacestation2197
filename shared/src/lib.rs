//! Shared logic for space station 2197.
//! You will mostly find definitions and systems capable of prediction here.
pub mod audio;
pub mod chat;
pub mod defines;
pub mod game;
pub mod meta;
pub mod networking;
pub mod utils;

pub mod placeholder;

use bevy::prelude::*;
use content::prelude::*;

use crate::{
    audio::AudioPlugin, chat::ChatPlugin, game::GamePlugin, meta::MetaPlugin,
    networking::NetworkingPlugin, placeholder::PlaceholderPlugin,
};

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InstanceState>()
            .add_plugins(ContentPlugin::default())
            .add_plugins(NetworkingPlugin)
            .add_plugins(AudioPlugin)
            .add_plugins(ChatPlugin)
            .add_plugins(GamePlugin)
            .add_plugins(MetaPlugin)
            .add_plugins(PlaceholderPlugin);
    }
}

/// Is the current instance hosting or listening to a game?
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstanceState {
    Hosting,
    #[default]
    Listening,
}
