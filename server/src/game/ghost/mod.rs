use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    game::{
        GameplaySystems,
        ghost::{CanSeeGhost, Ghost, GhostInput},
    },
    utils::filters::{MobFilter, PlayerFilter},
};

use crate::{
    game::mind::{Controlled, Controls},
    networking::ServerClientEntity,
    utils::{SpawnMethod, SpawnerCommandsExt},
};

pub(super) struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_visibility_filter::<Ghost>()
            .add_observer(add_see_ghost)
            .add_observer(remove_see_ghost)
            .add_systems(
                FixedUpdate,
                (read_input_ghosts,).in_set(GameplaySystems::Inputs),
            );
    }
}

fn read_input_ghosts(
    mut reader: MessageReader<FromClient<GhostInput>>,
    server_client: Res<ServerClientEntity>,
    mut commands: Commands,
    clients: Query<&Controls, PlayerFilter>,
    mobs: Query<&Transform, (MobFilter, Without<Ghost>)>,
) {
    for input in reader.read() {
        let client_entity = server_client.resolve(input.client_id);

        let Ok(controls) = clients.get(client_entity) else {
            continue;
        };

        let Ok(transform) = mobs.get(controls.entity()) else {
            // only non-ghost owning clients can ghost themselves!
            continue;
        };

        commands.spawn_player(
            client_entity,
            "ghost".to_string(),
            SpawnMethod::Position(transform.translation.xz()),
        );
    }
}

fn add_see_ghost(
    inserted: On<Add, Controlled>,
    mut commands: Commands,
    ghosts: Query<&Controlled, With<Ghost>>,
) {
    let Ok(controlled) = ghosts.get(inserted.entity) else {
        // not a ghost!
        return;
    };

    let player_entity = controlled.0;
    info!("adding can see ghost {player_entity:?}");
    commands.entity(player_entity).insert(CanSeeGhost);
}

fn remove_see_ghost(
    replaced: On<Replace, Controlled>,
    _commands: Commands,
    controlleds: Query<&Controlled>,
) {
    let _controlled = controlleds
        .get(replaced.entity)
        .expect("Should have controlled.");
    // commands.entity(controlled.0).remove::<CanSeeGhost>();
}
