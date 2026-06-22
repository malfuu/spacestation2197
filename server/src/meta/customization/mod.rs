use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    meta::customization::{PlayerSettings, SetCustomizationInput},
    utils::filters::PlayerFilter,
};

use crate::networking::ServerClientEntity;

pub(super) fn read_set_customization(
    mut reader: MessageReader<FromClient<SetCustomizationInput>>,
    server_client: Res<ServerClientEntity>,
    mut commands: Commands,
    players: Query<Entity, PlayerFilter>,
) {
    for input in reader.read() {
        let client_entity = server_client.resolve(input.client_id);

        let Ok(entity) = players.get(client_entity) else {
            warn!("No existing player entity for {client_entity:?}");
            continue;
        };

        let settings = input.0.clone();

        info!("Received customization {settings:?}");

        // TODO: check if customization is valid.
        commands.entity(entity).insert(PlayerSettings {
            character: settings,
        });
    }
}
