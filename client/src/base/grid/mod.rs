use bevy::prelude::*;

use content::prelude::*;
use tile_grid::{CHUNK_SIZE, Chunk, EntityChunk, chunk_and_local_to_world};

use shared::{defines::PROTOTYPE_TYPE_TILE, game::grid::TilePrototype};

pub(super) struct ClientGridPlugin;

impl Plugin for ClientGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_chunk_change)
            .add_observer(on_chunk_add);
    }
}

#[derive(Component)]
pub struct ChunkEntities {
    pub entities: EntityChunk,
}

/// Client-side Marker Component for entities representing tiles.
#[derive(Component)]
pub struct Tile {
    pub world_position: IVec2,
    pub local_position: UVec2,
}

fn on_chunk_add(add: On<Add, Chunk>, mut commands: Commands, chunks: Query<&Chunk>) {
    let chunk = chunks.get(add.entity).expect("Chunk should exist.");
    let chunk_pos = chunk.position();

    let mut entities = EntityChunk::from_value(Entity::PLACEHOLDER);
    let mut children = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);

    for x in 0..CHUNK_SIZE as u32 {
        for y in 0..CHUNK_SIZE as u32 {
            let local_position = UVec2::new(x, y);
            let world_position = chunk_and_local_to_world(chunk_pos, local_position);

            let tile_entity = commands
                .spawn((
                    Tile {
                        world_position,
                        local_position,
                    },
                    Transform::from_xyz(x as f32, 0.0, y as f32),
                ))
                .id();

            if let Some(slot) = entities.get_mut(local_position) {
                *slot = tile_entity;
            }

            children.push(tile_entity);
        }
    }

    let mut chunk_commands = commands.entity(add.entity);

    chunk_commands.insert((ChunkEntities { entities }, Visibility::Visible));
    for child in children {
        chunk_commands.add_child(child);
    }
}

fn on_chunk_change(
    chunks: Query<(Ref<Chunk>, &ChunkEntities)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<Prototypes>,
) {
    for (chunk, chunk_entities) in chunks {
        if !chunk.is_changed() {
            continue;
        }

        for x in 0..CHUNK_SIZE as u32 {
            for y in 0..CHUNK_SIZE as u32 {
                let local_pos = UVec2::new(x, y);

                let tile_entity = *chunk_entities
                    .entities
                    .get(local_pos)
                    .expect("Entity grid should be complete.");

                if tile_entity == Entity::PLACEHOLDER {
                    continue;
                }

                let Some(tile_slot) = chunk.tiles.get(local_pos) else {
                    continue;
                };

                match tile_slot {
                    Some(tile_tag) => {
                        if let Some(proto) =
                            registry.get::<TilePrototype>(PROTOTYPE_TYPE_TILE, tile_tag.clone())
                        {
                            commands.entity(tile_entity).insert(WorldAssetRoot(
                                asset_server.load(format!("{}#Scene0", proto.mesh)),
                            ));
                        }
                    }
                    None => {
                        commands
                            .entity(tile_entity)
                            .remove::<WorldAssetRoot>()
                            .despawn_children();
                    }
                }
            }
        }
    }
}
