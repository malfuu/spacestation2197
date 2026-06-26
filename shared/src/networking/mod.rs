use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy_renet::{
    RenetClient,
    netcode::{ClientAuthentication, NetcodeClientTransport},
    renet::ConnectionConfig,
};
use bevy_replicon_renet::{RenetChannelsExt, RepliconRenetPlugins};
use common::EntityTag;

pub const NETWORK_PROTOCOL_ID: u64 = 0;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliconPlugins)
            .add_plugins(RepliconRenetPlugins)
            .replicate::<Name>()
            .replicate::<Transform>()
            .replicate::<ChildOf>()
            .replicate_once::<EntityTag>();
    }
}

pub fn load_client(
    In((client_id, address, port, _password_opt)): In<(u64, IpAddr, u16, Option<String>)>,
    mut commands: Commands,
    channels: Res<RepliconChannels>,
) -> Result {
    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let client = bevy_renet::renet::RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let server_addr = SocketAddr::new(address, port);
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;

    // BUG: sending user data spits out weird errors on the server side?
    // let user_data: Option<[u8; 256]> = password_opt
    //     .map(|pass| pass.as_bytes().try_into().ok())
    //     .flatten();

    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: NETWORK_PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

    commands.insert_resource(RenetClient(client));
    commands.insert_resource(transport);

    Ok(())
}

#[cfg(feature = "steam")]
pub fn load_steam_client(
    In((_client_id, _address, _port, _password_opt)): In<(u64, IpAddr, u16, Option<String>)>,
    mut _commands: Commands,
    _channels: Res<RepliconChannels>,
) -> Result {
    todo!()
}
