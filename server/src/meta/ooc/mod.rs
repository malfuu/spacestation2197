use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    meta::{MetaSystems, ooc::PlayerOoc, player::PlayerName},
    utils::filters::PlayerFilter,
};

use crate::{networking::ServerClientEntity, utils::MessageCommandsExt};

pub(crate) struct OocPlugin;

impl Plugin for OocPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OocResource>()
            .add_systems(FixedUpdate, (read_oocs).in_set(MetaSystems::Inputs));
    }
}

#[derive(Resource, Debug)]
pub(crate) struct OocResource {
    pub(crate) enabled: bool,
}

impl Default for OocResource {
    fn default() -> Self {
        Self { enabled: true }
    }
}

fn read_oocs(
    mut reader: MessageReader<FromClient<PlayerOoc>>,
    server_client: Res<ServerClientEntity>,
    mut commands: Commands,
    resource: Res<OocResource>,
    ids: Query<&PlayerName, PlayerFilter>,
) {
    for msg in reader.read() {
        if !resource.enabled {
            continue; // dont forget to consume all incoming OOC messages regardless!
        }

        let client_entity = server_client.resolve(msg.client_id);

        let Ok(player_name) = ids.get(client_entity) else {
            continue;
        };

        let line = format!("OOC: {}: {}", player_name.get(), msg.0);

        commands.broadcast_chat_message(line);
    }
}
