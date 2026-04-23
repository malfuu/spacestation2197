//! saving and loading
use std::{collections::HashMap, fs, path::Path};

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ron::{
    from_str,
    ser::{PrettyConfig, to_string_pretty},
};
use serde::{Deserialize, Serialize};

use common::{EntityTag, PrototypeId};
use tile_grid::{CHUNK_AREA, CHUNK_SIZE, Chunk, Grid, chunk_and_local_to_world};

use content::{entity::PrototypeEntityCommandsExt, prelude::*};

use crate::{
    defines::CEILING_HEIGHT,
    game::{grid::GridCommandsExt, light::Light, physics::BOUNDARY_LAYER},
};

pub(super) struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_load_map);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChunkData {
    tiles: Vec<usize>,
    mixtures: Vec<usize>,
}

/// File representation of a grid
#[derive(Debug, Serialize, Deserialize)]
struct GridData {
    palette: Vec<PrototypeId>,
    palette_gas: Vec<PrototypeId>,
    chunks: HashMap<IVec2, ChunkData>,
}

impl GridData {
    fn from_world(world: &mut World) -> Self {
        let mut grid_query = world.query::<&Grid>();
        let grid = grid_query
            .single(world)
            .expect("There should be a single grid.");

        let mut palette = Vec::<String>::new();
        let mut palette_lookup = std::collections::HashMap::new();

        let mut chunks = HashMap::new();

        for (&chunk_pos, &chunk_entity) in &grid.chunks {
            let Some(chunk) = world.get::<Chunk>(chunk_entity) else {
                continue;
            };

            let mut tiles = Vec::with_capacity(CHUNK_AREA);

            for (_, tile_tag_opt) in chunk.tiles.iter_with_pos() {
                let index: u32 = if let Some(tile_tag) = tile_tag_opt {
                    let proto_id = tile_tag;

                    let raw_index = *palette_lookup.entry(proto_id.clone()).or_insert_with(|| {
                        let idx = palette.len() as u32;
                        palette.push(proto_id.clone());
                        idx
                    });

                    raw_index + 1
                } else {
                    0
                };

                tiles.push(index as usize);
            }

            chunks.insert(
                chunk_pos,
                ChunkData {
                    tiles,
                    mixtures: vec![],
                },
            );
        }

        Self {
            palette,
            palette_gas: vec![],
            chunks,
        }
    }

    fn spawn_world_tiles(&self, commands: &mut Commands) {
        for (chunk_position, chunk) in self.chunks.iter() {
            for (idx, palette_index) in chunk.tiles.iter().enumerate() {
                if *palette_index == 0 {
                    continue;
                }

                let proto_id = self
                    .palette
                    .get(*palette_index - 1)
                    .expect("Wrong palette index!");

                let local_x = idx % CHUNK_SIZE;
                let local_y = idx / CHUNK_SIZE;

                let local_position = UVec2::new(local_x as u32, local_y as u32);
                let world_position = chunk_and_local_to_world(*chunk_position, local_position);

                commands.spawn_tile(proto_id.clone(), world_position);
            }
        }
    }
}

/// How is an entity saved in map info.
// type SavedEntity = (String, Transform);
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedEntity {
    entity: Entity,
    tag: PrototypeId,
    transform: Transform,
}

fn save_entities(world: &mut World) -> Vec<SavedEntity> {
    let mut entities = Vec::new();

    for (entity, tag, transform) in world
        .query::<(Entity, &EntityTag, &Transform)>()
        .iter(world)
    {
        let prototype_registry = world.resource::<Prototypes>();
        if !prototype_registry.contains(PROTOTYPE_CATEGORY_ENTITY, &tag.0) {
            continue;
        }

        let saved_entity = SavedEntity {
            entity,
            tag: tag.0.clone(),
            transform: *transform,
        };

        entities.push(saved_entity);
    }

    entities
}

fn write_entities(entities: &Vec<SavedEntity>, commands: &mut Commands) {
    for SavedEntity {
        entity: _entity,
        tag,
        transform,
    } in entities
    {
        commands.spawn_prototype(tag.clone(), *transform);
    }
}

/// imagine bevy's DynamicScene, but ours.
#[derive(Debug, Serialize, Deserialize)]
pub struct MapInformation {
    name: String,
    grid_data: GridData,
    entities: Vec<SavedEntity>,
}

impl MapInformation {
    /// Stop-the-world saving of map state.
    pub fn from_world(name: &str, world: &mut World) -> Option<Self> {
        if !is_map_name_valid(name) {
            return None;
        }

        let grid_data = GridData::from_world(world);
        let entities = save_entities(world);
        // TODO: save mixtures

        let map = Self {
            name: name.to_owned(),
            grid_data,
            entities,
        };

        Some(map)
    }

    pub fn write_to_world(&self, commands: &mut Commands, _prototype_registry: &Prototypes) {
        self.grid_data.spawn_world_tiles(commands);
        write_entities(&self.entities, commands);
    }

    pub fn serialize(&self) -> Option<String> {
        let pretty_config = PrettyConfig::new().depth_limit(4);

        let ron_string = match to_string_pretty(&self, pretty_config) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to serialize to RON: {}", e);
                return None;
            }
        };

        Some(ron_string)
    }
}

pub fn load_map_information(grid_name: &String) -> Option<MapInformation> {
    let ron_path = Path::new("assets/grids").join(format!("{grid_name}.ron"));

    if !ron_path.exists() {
        panic!("Grid file '{:?}' does not exist", ron_path);
    }

    let content = match fs::read_to_string(&ron_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to read {:?}: {}", ron_path, e);
            return None;
        }
    };

    let map_info: MapInformation = match from_str(&content) {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to deserialize RON from {:?}: {}", ron_path, e);
            return None;
        }
    };

    Some(map_info)
}

pub fn load_grid(
    In(grid_name): In<String>,
    mut commands: Commands,
    prototype_registry: Res<Prototypes>,
) {
    info!("Loading grid with name {}", grid_name);

    let Some(map_info) = load_map_information(&grid_name) else {
        error!("No map with name {}", grid_name);
        commands.write_message(AppExit::error());
        return;
    };

    commands.spawn((Replicated, Grid::new()));
    map_info.write_to_world(&mut commands, &prototype_registry);
}

pub fn get_list_of_grids() -> Vec<String> {
    fs::read_dir("assets/grids")
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_file())
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ron"))
                .filter_map(|entry| {
                    entry
                        .path()
                        .file_stem() // we remove the extension here btw
                        .and_then(|s| s.to_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Event)]
pub struct LoadMap(pub String);

pub fn is_map_name_valid(name: &str) -> bool {
    // very simple implementation
    name.is_ascii() && !name.is_empty() && name.len() < 50
}

fn on_load_map(load_map: On<LoadMap>, mut commands: Commands) {
    commands.run_system_cached_with(load_grid, load_map.0.clone());

    commands.spawn((
        Replicated,
        Transform::from_xyz(0., 1.25, 0.),
        Light {
            color: Srgba::gray(1.0),
            intensity: 10_000.0,
            range: 16.0,
        },
    ));

    // floor
    commands.spawn((
        Replicated,
        Transform::IDENTITY,
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        BOUNDARY_LAYER,
    ));

    // ceiling
    commands.spawn((
        Replicated,
        Transform::from_xyz(0., CEILING_HEIGHT, 0.),
        RigidBody::Static,
        Collider::half_space(Vec3::NEG_Y),
        BOUNDARY_LAYER,
    ));

    let boundaries = [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z];

    for dir in boundaries {
        commands.spawn((
            Replicated,
            Transform::from_translation(dir * -128.0),
            RigidBody::Static,
            Collider::half_space(dir),
            BOUNDARY_LAYER,
        ));
    }
}
