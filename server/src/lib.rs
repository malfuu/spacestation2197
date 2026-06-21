pub mod audio;
pub mod networking;
pub mod utils;

mod game;
mod meta;

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use atmos_simulation::prelude::*;

use common::EntityTag;
use shared::{
    SharedPlugin,
    game::persistence::LoadMap,
    meta::{
        manager::Manager,
        round::{RoundState, SetStartTimer},
    },
    utils::ServerSettings,
};

use crate::{
    audio::ServerAudioPlugin,
    game::ServerGamePlugin,
    meta::ServerMetaPlugin,
    networking::{ServerNetworkingPlugin, load_server},
};

pub const SERVER_CONFIG_FILENAME: &str = "server_config.toml";

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<SharedPlugin>());

        app.add_plugins(ServerAudioPlugin)
            .add_plugins(ServerNetworkingPlugin)
            .add_plugins(ServerGamePlugin)
            .add_plugins(ServerMetaPlugin)
            .add_observer(replicate_new_entities);
    }
}

fn replicate_new_entities(on: On<Add, EntityTag>, mut commands: Commands) {
    commands.entity(on.entity).insert(Replicated);
}

/// Starts hosting a game instance locally
/// In practice it's only used by the server.
pub fn start_game_instance(
    mut commands: Commands,
    mut atmos_resource: ResMut<AtmosphericsResource>,
) {
    let settings = ServerSettings::load_from_file(SERVER_CONFIG_FILENAME);
    commands.insert_resource(settings.clone());
    commands.trigger(LoadMap(settings.map_name.clone()));
    atmos_resource.enabled = settings.atmos_enabled;

    commands.spawn((
        Name::new("Manager"),
        Replicated,
        Manager,
        RoundState::Starting,
        settings.gamemode.clone(),
    ));
    commands.trigger(SetStartTimer);

    // open server & connections
    commands.run_system_cached(load_server);
}
