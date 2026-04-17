use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    meta::customization::{PlayerSettings, SetCustomizationInput},
    utils::filters::PlayerFilter,
};

pub(super) fn read_set_customization(
    mut reader: MessageReader<FromClient<SetCustomizationInput>>,
    mut commands: Commands,
    players: Query<Entity, PlayerFilter>,
) {
    for input in reader.read() {
        let ClientId::Client(client_entity) = input.client_id else {
            continue;
        };

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
