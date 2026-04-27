use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use mlua::prelude::*;

use common::PrototypeId;
use content::prelude::*;
use tile_grid::{
    BooleanChunk, CHUNK_SIZE, Chunk, Grid, LocalTilePosition, world_to_chunk_and_local,
};

use atmos_simulation::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{defines::PROTOTYPE_TYPE_TILE, game::GameplaySystems, utils::filters::ChunkFilter};

pub(super) struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.prototype::<TilePrototype>(PROTOTYPE_TYPE_TILE, tile_parser)
            .prototype_component::<Subfloor>()
            .replicate::<Grid>()
            .replicate::<Chunk>()
            .add_systems(
                FixedUpdate,
                (update_chunk_properties, update_grid_colliders).in_set(GameplaySystems::Logic),
            )
            .add_observer(on_grid_despawn)
            .add_observer(on_chunk_add);
    }
}

/// Marks an entity as being underfloor, possibly hidden by tiles.
#[derive(Component, Reflect, Clone, Default, Serialize, Deserialize)]
#[reflect(Component, Clone)]
pub struct Subfloor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePrototype {
    pub id: PrototypeId,
    /// Becomes part of chunk collider
    pub is_solid: bool,
    /// Can eyes see through it?
    pub is_opaque: bool,
    /// Is tile considered space
    pub is_space: bool,
    /// Does it show subfloor entities?
    pub is_subfloor: bool,
    pub mesh: String,
}

fn tile_parser(_: &Lua, table: LuaTable) -> ParseResult {
    let proto = TilePrototype {
        id: table.get("id")?,
        is_solid: table.get("is_solid").unwrap_or(false),
        is_opaque: table.get("is_opaque").unwrap_or(false),
        is_space: table.get("is_space").unwrap_or(false),
        is_subfloor: table.get("is_subfloor").unwrap_or(false),
        mesh: table.get("mesh")?,
    };

    Ok(Box::new(proto))
}

type ChunkPropertyQueryData<'a> = (
    Ref<'a, Chunk>,
    Mut<'a, SubfloorChunk>,
    Mut<'a, SpaceChunk>,
    Mut<'a, ImpermeableChunk>,
    Mut<'a, SolidChunk>,
);

pub(super) fn update_chunk_properties(
    mut chunks: Query<ChunkPropertyQueryData, With<Chunk>>,
    registry: Res<Prototypes>,
) {
    for (chunk, mut subfloor, mut space, mut impermeable, mut solid) in &mut chunks {
        if !chunk.is_changed() {
            continue;
        }

        for x in 0..CHUNK_SIZE as u32 {
            for y in 0..CHUNK_SIZE as u32 {
                let local_pos = LocalTilePosition::new(x, y);

                let mut is_subfloor = false;
                let mut is_space = true;
                let mut is_solid = false;

                if let Some(Some(tile_tag)) = chunk.tiles.get(local_pos)
                    && let Some(proto) =
                        registry.get::<TilePrototype>(PROTOTYPE_TYPE_TILE, tile_tag.clone())
                {
                    is_subfloor = proto.is_solid;
                    is_space = proto.is_space;
                    is_solid = proto.is_solid;
                }

                if let Some(val) = subfloor.0.get_mut(local_pos) {
                    *val = is_subfloor;
                }
                if let Some(val) = space.0.get_mut(local_pos) {
                    *val = is_space;
                }
                if let Some(val) = impermeable.0.get_mut(local_pos) {
                    *val = is_solid;
                }
                if let Some(val) = solid.0.get_mut(local_pos) {
                    *val = is_solid;
                }
            }
        }
    }
}

/// what tiles have their subfloor exposed and which do not.
#[derive(Component, Default)]
pub struct SubfloorChunk(pub BooleanChunk);

/// what tiles are solid which are not
#[derive(Component, Default)]
pub struct SolidChunk(pub BooleanChunk);

pub(super) fn on_grid_despawn(
    despawn: On<Despawn, Grid>,
    mut commands: Commands,
    grid: Query<&Grid>,
) {
    let Ok(grid) = grid.get(despawn.entity) else {
        return;
    };

    for chunk_entity in grid.chunks.values() {
        commands.entity(*chunk_entity).despawn();
    }
}

pub(super) fn on_chunk_add(add: On<Add, Chunk>, mut commands: Commands, chunks: Query<&Chunk>) {
    let chunk = chunks.get(add.entity).expect("Chunk should exist.");
    let chunk_position = chunk.position().as_vec2();
    let translation = Vec3::new(chunk_position.x, 0.0, chunk_position.y) * CHUNK_SIZE as f32;

    commands.entity(add.entity).insert((
        Transform::from_translation(translation),
        Replicated,
        RigidBody::Static,
        ChunkMixtures::default(),
        Flows::default(),
        SubfloorChunk::default(),
        SolidChunk::default(),
        atmos_simulation::chunk::ChunkDeltas::default(),
        SpaceChunk::default(),
        ImpermeableChunk::default(),
        atmos_simulation::excited::ProcessedTick::default(),
    ));
}

/// create a tile locally
pub trait GridCommandsExt {
    fn delete_tile(&mut self, world_position: IVec2) -> &mut Self;
    fn spawn_tile(&mut self, prototype: impl Into<String>, world_position: IVec2) -> &mut Self;
}

impl GridCommandsExt for Commands<'_, '_> {
    fn delete_tile(&mut self, world_position: IVec2) -> &mut Self {
        self.queue(move |world: &mut World| {
            let (chunk_position, local_position) = world_to_chunk_and_local(world_position);

            let grid = world
                .query::<&Grid>()
                .single(world)
                .expect("A grid should exist.");

            let Some(&chunk_entity) = grid.chunks.get(&chunk_position) else {
                return;
            };
            let Some(mut chunk) = world.get_mut::<Chunk>(chunk_entity) else {
                return;
            };
            let Some(tile_slot) = chunk.tiles.get_mut(local_position) else {
                return;
            };

            *tile_slot = None;
        });

        self
    }

    fn spawn_tile(&mut self, prototype: impl Into<String>, world_position: IVec2) -> &mut Self {
        let prototype = prototype.into();

        self.queue(move |world: &mut World| {
            let (chunk_position, local_position) = world_to_chunk_and_local(world_position);

            let chunk_entity = {
                let grid = world
                    .query::<&Grid>()
                    .single(world)
                    .expect("A grid should exist.");

                if let Some(entity) = grid.chunks.get(&chunk_position) {
                    *entity
                } else {
                    let new_entity = world.spawn(Chunk::new(chunk_position)).id();

                    let mut grid_mut = world
                        .query::<&mut Grid>()
                        .single_mut(world)
                        .expect("A grid should exist.");

                    grid_mut.add(chunk_position, new_entity);

                    new_entity
                }
            };

            if let Some(mut chunk) = world.get_mut::<Chunk>(chunk_entity)
                && let Some(tile_slot) = chunk.tiles.get_mut(local_position)
            {
                *tile_slot = Some(prototype);
            }
        });

        self
    }
}

fn update_grid_colliders(
    mut commands: Commands,
    chunks: Query<(Entity, Ref<SolidChunk>), ChunkFilter>,
) {
    for (chunk_entity, solid_chunk) in chunks.iter() {
        if !solid_chunk.is_changed() {
            continue;
        }

        let colliders = build_colliders(&solid_chunk.0);

        let mut entity_commands = commands.entity(chunk_entity);
        if let Some(collider) = colliders {
            entity_commands.insert(collider);
        } else {
            entity_commands.remove::<Collider>();
        }
    }
}

fn build_colliders(grid: &BooleanChunk) -> Option<Collider> {
    let mut shapes = Vec::new();

    for (pos, cell) in grid.iter_with_pos() {
        if !cell {
            continue;
        }

        let collider_position = Vec3::new(pos.x as f32 + 0.5, 1.25, pos.y as f32 + 0.5);

        shapes.push((
            collider_position,
            Quat::IDENTITY,
            Collider::cuboid(1.0, 2.5, 1.0),
        ));
    }

    if shapes.is_empty() {
        return None;
    }

    Some(Collider::compound(shapes))
}
