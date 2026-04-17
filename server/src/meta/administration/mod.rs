use atmos::engine::AtmosphericsResource;
use avian3d::prelude::{Physics, PhysicsTime};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_renet::RenetServer;
use bevy_replicon::{prelude::*, shared::backend::connected_client::NetworkId};

use shared::{
    meta::{
        MetaSystems,
        administration::{AdminCommandMessage, Administrator},
        round::{EndRoundEvent, RoundStartedEvent, RoundState},
    },
    utils::filters::PlayerFilter,
};

use crate::{meta::ooc::OocResource, utils::MessageCommandsExt};

pub const MINIMUM_TICKS: u32 = 1;
pub const MAXIMUM_TICKS: u32 = 120;

pub(super) struct ServerAdministrationPlugin;

impl Plugin for ServerAdministrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (read_admin_commands).in_set(MetaSystems::Inputs),
        );
    }
}

pub(crate) fn admin_message(In(_network_id): In<NetworkId>) {
    // let _client_id: ClientId = todo!();
}

pub(crate) fn kick_player(In(network_id): In<NetworkId>, mut server: ResMut<RenetServer>) {
    info!("kicking player {network_id:?}");
    server.disconnect(network_id.get());
}

pub(crate) fn respawn_player(In(_network_id): In<NetworkId>) {}

pub(crate) fn is_admin(client_id: ClientId, admins: &Query<&Administrator, PlayerFilter>) -> bool {
    match client_id {
        ClientId::Client(entity) => admins.contains(entity),
        ClientId::Server => {
            true // assume server is always admin
        }
    }
}

#[derive(SystemParam)]
struct AdminCommandsData<'w> {
    ooc_resource: ResMut<'w, OocResource>,
    time_fixed: ResMut<'w, Time<Fixed>>,
    time_physics: ResMut<'w, Time<Physics>>,
    atmos_resource: ResMut<'w, AtmosphericsResource>,
}

fn read_admin_commands(
    mut reader: MessageReader<FromClient<AdminCommandMessage>>,
    mut commands: Commands,
    admins: Query<&Administrator, PlayerFilter>,
    round_state: Single<(Entity, &RoundState)>,
    data: AdminCommandsData,
) {
    let AdminCommandsData {
        mut ooc_resource,
        mut time_fixed,
        mut time_physics,
        mut atmos_resource,
    } = data;

    for msg in reader.read() {
        if !is_admin(msg.client_id, &admins) {
            info!("Received admin command from non admin {:?}", msg.client_id);
            continue;
        }

        match &msg.message {
            AdminCommandMessage::SetTps(set_ticks) => {
                let constrained_ticks = (*set_ticks).clamp(MINIMUM_TICKS, MAXIMUM_TICKS);
                info!("Setting ticks to {}", constrained_ticks);
                time_fixed.set_timestep_hz(constrained_ticks as f64);
            }
            AdminCommandMessage::SetAtmos(desired) => {
                atmos_resource.enabled = *desired;
            }
            AdminCommandMessage::SetPhysics(desired) => {
                if *desired {
                    time_physics.unpause();
                } else {
                    time_physics.pause();
                }
            }
            AdminCommandMessage::SetGameplay(_desired) => {
                warn!("TODO: PlayerAdminCommands::SetGameplay");
            }
            AdminCommandMessage::ForceStartRound => {
                commands.trigger(RoundStartedEvent);
            }
            AdminCommandMessage::ForceEndRound => {
                commands.trigger(EndRoundEvent);
            }
            AdminCommandMessage::Shutdown => {
                info!("Received Shutdown command from {:?}!", msg.client_id);
                commands.write_message(AppExit::Success);
            }
            AdminCommandMessage::AdminMessage(network_id) => {
                commands.run_system_cached_with(admin_message, *network_id);
            }
            AdminCommandMessage::Kick(network_id) => {
                commands.run_system_cached_with(kick_player, *network_id);
            }
            AdminCommandMessage::Ban(_network_id) => {
                warn!("TODO: implementing banning!"); // TODO
            }
            AdminCommandMessage::Respawn(network_id) => {
                commands.run_system_cached_with(respawn_player, *network_id);
            }
            AdminCommandMessage::SetOoc(desired) => {
                ooc_resource.enabled = *desired;
                commands.broadcast_chat_message(format!("OOC has been toggled {desired}."));
            }
            AdminCommandMessage::SetGamemode(gamemode) => {
                let (round_entity, state) = *round_state;

                match state {
                    RoundState::Starting => {
                        commands.entity(round_entity).insert(gamemode.clone());
                    }
                    _ => {
                        info!("receiving SetGamemode({gamemode:?}) after round has started.");
                    }
                }
            }
        }
    }
}
