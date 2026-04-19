use avian3d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};

use atmos_simulation::chunk::Mixtures;
use grid::{Grid, world_to_chunk_and_local};

use shared::{
    game::{GameplaySystems, physics::NORMAL_LAYER},
    utils::filters::{ChunkFilter, ItemFilter, MobFilter},
};

pub(super) struct ServerAtmosPlugin;

impl Plugin for ServerAtmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, wind_push.in_set(GameplaySystems::Logic));
    }
}

#[derive(QueryFilter)]
struct PushableFilter {
    or: Or<(MobFilter, ItemFilter)>,
}

fn wind_push(
    mut entities: Query<(&Transform, &CollisionLayers, Forces), PushableFilter>,
    grid: Single<&Grid>,
    chunks: Query<&Mixtures, ChunkFilter>,
) {
    for (transform, layers, mut forces) in entities.iter_mut() {
        if !layers.interacts_with(NORMAL_LAYER) {
            continue;
        }

        let position = transform.translation.floor().as_ivec3().xz();
        let (chunk_position, local_position) = world_to_chunk_and_local(position);

        let Some(chunk_entity) = grid.get(chunk_position) else {
            continue;
        };

        let atmos = chunks.get(chunk_entity).expect("Chunk should exist.");
        let wind_force = atmos
            .flows()
            .get(local_position)
            .expect("Local position is valid.");
        if wind_force.length_squared() < 0.01 {
            continue;
        }

        // info!("chunk: {chunk_position:?}, local: {local_position:?}, pos: {position:?}");
        // info!("apply force of {wind_force:?}");
        let wind_force = Vec3::new(wind_force.x, 0., wind_force.y);

        forces.apply_force(wind_force);
    }
}
