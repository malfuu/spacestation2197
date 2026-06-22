use std::{
    net::{Ipv4Addr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;
use bevy_renet::{
    RenetServer,
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::ConnectionConfig,
};
use bevy_replicon::{prelude::*, shared::backend::connected_client::NetworkId};

use bevy_replicon_renet::RenetChannelsExt;
use shared::{
    game::player::Player,
    meta::{
        MetaSystems,
        administration::Administrator,
        customization::PlayerSettings,
        player::{Ping, PlayerName},
    },
    networking::NETWORK_PROTOCOL_ID,
    utils::ServerSettings,
};

pub(super) struct ServerNetworkingPlugin;

impl Plugin for ServerNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.sync_related_entities::<ChildOf>()
            .init_resource::<ServerClientEntity>()
            .add_systems(FixedUpdate, update_ping.in_set(MetaSystems::Logic))
            .add_observer(on_joining_client)
            .add_observer(listen_leaving_client);
    }
}

/// In case the server is also a participating client. (e.g. player hosting co-op).
/// The inner entity will point to a backend created client entity.
#[derive(Resource, Default, Deref)]
pub struct ServerClientEntity(Option<Entity>);

impl ServerClientEntity {
    /// Resolves the client entity from a `ClientId`
    /// Falls back to the server's local client entity if it's `ClientId::Server`, panics if no
    /// local client is present.
    pub fn resolve(&self, client_id: ClientId) -> Entity {
        client_id
            .entity()
            .or(self.0)
            .expect("Server origin without server client preent!")
    }
}

fn on_joining_client(
    add: On<Add, ConnectedClient>,
    mut commands: Commands,
    settings: Res<ServerSettings>,
    mut _server: ResMut<RenetServer>,
    netcode_server: Res<NetcodeServerTransport>,
    clients: Query<&NetworkId>,
) {
    let client_id = clients
        .get(add.entity)
        .expect("Client Entity should have NetworkId!");

    let addr = netcode_server
        .client_addr(client_id.get())
        .expect("Client should have an address.");
    info!("Client {:?} joined from {:?}!", add.entity, addr);

    if !is_password_valid(
        &settings.password,
        &netcode_server.user_data(client_id.get()),
        add.entity,
    ) {
        // NOTE: we arent doing password enforcement yet.
        //     server.disconnect(client_id.get());
        //     return;
    }

    let mut e = commands.entity(add.entity);
    let player_name = PlayerName::random();
    let player_settings = PlayerSettings::random();

    e.insert((
        Replicated,
        AuthorizedClient,
        Player,
        player_name,
        player_settings.clone(),
        Ping::default(),
    ));

    let is_local = { addr.ip().is_loopback() };

    if is_local && settings.make_local_admin {
        info!("Client {:?} made into local administrator!", add.entity);
        e.insert(Administrator);
    }
}

fn is_password_valid(
    server_password: &Option<String>,
    user_data: &Option<[u8; 256]>,
    client: Entity,
) -> bool {
    let Some(server_password) = &server_password else {
        return true;
    };

    let Some(user_data) = user_data else {
        info!("Client {:?} used the wrong password!", client);
        return false;
    };

    let Ok(user_password) = String::from_utf8(user_data.to_vec()) else {
        info!("Client {:?} inserted a non UTF-8 password!", client);
        return false;
    };

    if *server_password != user_password {
        info!("Client {:?} used the wrong password!", client);
        return false;
    }

    true
}

fn listen_leaving_client(remove: On<Despawn, ConnectedClient>) {
    info!("Client {:?} gone!", remove.entity);
}

fn update_ping(mut commands: Commands, query: Query<(Entity, &ConnectedClientStats)>) {
    for (entity, stats) in query.iter() {
        commands.entity(entity).insert(Ping {
            rtt: (stats.rtt as f32) * 1000.0,
        });
    }
}

pub fn load_server(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    server_settings: Res<ServerSettings>,
) -> Result {
    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let server = bevy_renet::RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, server_settings.port))?;
    let server_config = ServerConfig {
        current_time,
        max_clients: server_settings.max_players,
        protocol_id: NETWORK_PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: Default::default(),
    };

    let transport = NetcodeServerTransport::new(server_config, socket)?;

    commands.insert_resource(server);
    commands.insert_resource(transport);

    Ok(())
}
