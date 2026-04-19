use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_replicon::prelude::*;

use atmos_primitives::prelude::*;
use atmos_simulation::prelude::*;
use common::EntityTag;
use content::prelude::*;
use grid::{Grid, world_to_chunk_and_local};

use shared::{
    game::{
        GameplaySystems,
        grid::GridCommandsExt,
        placement::Placement,
        sandbox::{SandboxCommands, Sandboxer},
    },
    utils::filters::PlayerFilter,
};

pub(super) struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (read_sandbox_commands).in_set(GameplaySystems::Inputs),
        );
    }
}

pub(crate) fn is_sandboxer(client_id: ClientId, admins: &Query<&Sandboxer, PlayerFilter>) -> bool {
    match client_id {
        ClientId::Client(entity) => admins.contains(entity),
        ClientId::Server => {
            true // assume server is always admin
        }
    }
}

#[derive(SystemParam)]
struct SandboxData<'w, 's> {
    gas_list: Res<'w, GasList>,
    mixture_list: Res<'w, MixtureList>,

    admins: Query<'w, 's, &'static Sandboxer, PlayerFilter>,
    entities: Query<'w, 's, Entity, With<EntityTag>>,

    grid: Single<'w, 's, &'static Grid>,
    chunks: Query<'w, 's, &'static mut Mixtures>,
}

fn read_sandbox_commands(
    mut reader: MessageReader<FromClient<SandboxCommands>>,
    mut commands: Commands,
    mut sandbox_data: SandboxData,
) {
    for msg in reader.read() {
        if !is_sandboxer(msg.client_id, &sandbox_data.admins) {
            info!(
                "Received sandbox command from someone without permission: {:?}",
                msg.client_id
            );
            continue;
        }

        match &msg.message {
            SandboxCommands::Place(placement) => match placement {
                Placement::Entity { entity, position } => {
                    commands.spawn_prototype(
                        entity.to_string(),
                        Transform::from_xyz(position.x, 0., position.y),
                    );
                }
                Placement::Tile { tile, start, end } => {
                    for x in start.x.min(end.x)..=start.x.max(end.x) {
                        for y in start.y.min(end.y)..=start.y.max(end.y) {
                            commands.spawn_tile(tile.clone(), IVec2::new(x, y));
                        }
                    }
                }
            },
            SandboxCommands::EraseEntity(entity) => {
                if !sandbox_data.entities.contains(*entity) {
                    warn!("Received non-entity delete for {entity:?}");
                    continue;
                }
                commands.entity(*entity).despawn();
            }
            SandboxCommands::EraseTile(position) => {
                commands.delete_tile(*position);
            }
            SandboxCommands::SetMixture(mixture_id, world_position) => {
                let (chunk_position, local_position) = world_to_chunk_and_local(*world_position);

                let Some(chunk_entity) = sandbox_data.grid.chunks.get(&chunk_position) else {
                    continue;
                };

                let mut chunk = sandbox_data
                    .chunks
                    .get_mut(*chunk_entity)
                    .expect("chunk should exist with air");

                let mix = chunk
                    .mixtures_mut()
                    .get_mut(local_position)
                    .expect("pos should be within grid");

                match mixture_id {
                    Some(id) => {
                        let Some(blueprint) = sandbox_data.mixture_list.get(id.as_str()) else {
                            continue;
                        };
                        blueprint.apply_to(mix, &sandbox_data.gas_list);
                    }
                    None => {
                        mix.clear();
                    }
                }
            }
        }
    }
}
