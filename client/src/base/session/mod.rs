//! Manages session to a game server
mod ui;

use std::{
    net::{IpAddr, Ipv4Addr},
    time::SystemTime,
};

use bevy::prelude::*;
use bevy_egui::prelude::*;
use bevy_replicon::{prelude::*, shared::backend::connected_client::NetworkId};

use bevy_renet::{RenetClient, netcode::NetcodeClientTransport, renet::DisconnectReason};
use shared::{defines::DEFAULT_LISTEN_PORT, game::player::Player, networking::load_client};

use crate::base::{
    session::ui::{delete_loading_background, draw_loading_background, ui_loading},
    states::AppState,
};

pub(super) struct ClientSessionPlugin;

impl Plugin for ClientSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SessionState>()
            .add_systems(
                EguiPrimaryContextPass,
                ui_loading.run_if(in_state(SessionState::Connecting)),
            )
            .add_systems(OnEnter(SessionState::Connecting), draw_loading_background)
            .add_systems(OnExit(SessionState::Connecting), delete_loading_background)
            .add_systems(OnEnter(ClientState::Connected), on_connected)
            .add_systems(OnEnter(ClientState::Disconnected), on_disconnected)
            .add_observer(on_join_game)
            .add_observer(on_disconnect)
            .add_observer(on_add_player);
    }
}

#[derive(SubStates, PartialEq, Eq, Hash, Default, Debug, Clone)]
#[source(AppState = AppState::InGame)]
pub enum SessionState {
    #[default]
    Connecting,
    Playing,
}

#[derive(Resource, Debug)]
pub struct ClientNetworkId(u64);

/// Marker for [`shared::game::player::Player`] to indicate
/// which player this client is.
#[derive(Component, Debug)]
pub struct ThisPlayer;

/// Triggers a client to join a game.
#[derive(Event, Debug)]
pub struct JoinGame {
    pub address: IpAddr,
    pub port: u16,
    pub password: Option<String>,
}

/// Triggers a client to disconnect.
#[derive(Event, Debug)]
pub struct Disconnect;

impl Default for JoinGame {
    fn default() -> Self {
        Self {
            address: Ipv4Addr::LOCALHOST.into(),
            port: DEFAULT_LISTEN_PORT,
            password: None,
        }
    }
}

fn on_connected(mut commands: Commands) {
    commands.set_state(SessionState::Playing);
}

fn on_disconnected(mut commands: Commands, client: Option<Res<RenetClient>>) {
    let Some(client) = client else {
        return;
    };

    commands.remove_resource::<ClientNetworkId>();
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
    commands.set_state(AppState::Menu);

    let disconnect_reason = client
        .disconnect_reason()
        .expect("There should be a reason.");
    info!("Disconnected! Reason {:?}", disconnect_reason);

    if !matches!(disconnect_reason, DisconnectReason::DisconnectedByClient) {
        // TODO pop-up about reason
    }
}

fn on_join_game(join_game: On<JoinGame>, mut commands: Commands) {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("shouild have time");
    let client_id = current_time.as_millis() as u64;
    commands.insert_resource(ClientNetworkId(client_id));

    commands.set_state(AppState::InGame);
    commands.run_system_cached_with(
        load_client,
        (
            client_id,
            join_game.address,
            join_game.port,
            join_game.password.clone(),
        ),
    );
}

fn on_disconnect(
    _: On<Disconnect>,
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    if !matches!(**state, AppState::InGame) {
        warn!("Received disconnect while not in game!");
        return;
    }

    let mut client = client.expect("RenetClient should exist while playing!");
    client.disconnect();
}

fn on_add_player(
    trigger: On<Add, Player>,
    mut commands: Commands,
    query: Query<&NetworkId>,
    client_id: Res<ClientNetworkId>,
) {
    let entity = trigger.entity;
    let Ok(player_network_id) = query.get(entity) else {
        return;
    };

    if player_network_id.get() == client_id.0 {
        commands.entity(entity).insert(ThisPlayer);
    }
}
