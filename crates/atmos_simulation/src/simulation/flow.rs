use bevy::{ecs::query::QueryFilter, prelude::*};
use std::collections::HashSet;
use tile_grid::{CHUNK_SIZE, Grid};

use atmos_primitives::{
    BASE_DIFFUSION_COEFFICIENT, equations::ideal_gas_law_moles_wide, prelude::*,
};
use wide::f32x16;

use crate::{
    chunk::{
        ChunkCached, ChunkMixtures, Flows, INTERNAL_EDGES_LENGTH, ImpermeableChunk, SpaceChunk,
    },
    simulation::{AtmosSchedule, AtmosStepSystems},
    tile_mixture::{CachedTile, TileMixtureViewMut},
};

const NEWTONS_PER_MOLE: f32 = 1000.0; // this is arbitrary btw

pub(super) struct FlowSimulation;

impl Plugin for FlowSimulation {
    fn build(&self, app: &mut App) {
        app.add_systems(
            AtmosSchedule,
            (
                reset_flows,
                cache_mixtures,
                apply_internal_exchanges,
                apply_external_exchanges,
                cull_mixtures,
                update_space_clear,
            )
                .chain()
                .in_set(AtmosStepSystems::FlowPhase),
        );
    }
}

fn reset_flows(active_chunks: Query<Mut<Flows>>) {
    for mut flows in active_chunks {
        flows.iter_mut().for_each(|f| *f = Vec2::ZERO);
    }
}

fn cache_mixtures(
    gas_list: Res<GasList>,
    mut active_chunks: Query<(&ChunkMixtures, &mut ChunkCached)>,
) {
    let molar_heat_capacities = gas_list.get_molar_heat_capacities();
    for (mixtures, mut cached) in &mut active_chunks {
        for y in 0..CHUNK_SIZE as u32 {
            for x in 0..CHUNK_SIZE as u32 {
                let pos = uvec2(x, y);
                let view = mixtures.tile_view(pos).expect("pos valid");

                let temperature = view.temperature(molar_heat_capacities);
                let partial_pressures = view.partial_pressures(&gas_list);
                let heat_capacities = view.partial_heat_capacities(molar_heat_capacities);

                let c = cached.get_mut(pos).expect("pos valid");
                c.temperature = temperature;
                c.partial_pressures = partial_pressures;
                c.heat_capacities = heat_capacities;
            }
        }
    }
}

fn apply_internal_exchanges(
    gas_list: Res<GasList>,
    mut active_chunks: Query<(&mut ChunkMixtures, &mut Flows, &ChunkCached)>,
) {
    for (mut mixtures, mut flows, cached) in &mut active_chunks {
        // horizontal exchanges
        for y in 0..CHUNK_SIZE {
            let y = y as u32;
            for x in 0..INTERNAL_EDGES_LENGTH {
                let x = x as u32;
                let from_pos = uvec2(x, y);
                let to_pos = uvec2(x + 1, y);

                let [mix_from, mix_to] = mixtures
                    .tile_view_many_mut([from_pos, to_pos])
                    .expect("pos valid");
                let [cached_from, cached_to] =
                    cached.get_many([from_pos, to_pos]).expect("pos valid");
                let exchanged_amounts =
                    exchange(&gas_list, mix_from, mix_to, cached_from, cached_to);
                let exchanged_sum = exchanged_amounts.reduce_add();

                let [flow_from, flow_to] =
                    flows.get_many_mut([from_pos, to_pos]).expect("pos valid");

                let flow_direction = Vec2::X;
                let force = exchanged_sum * NEWTONS_PER_MOLE;
                *flow_from += flow_direction * force;
                *flow_to += flow_direction * force;
            }
        }

        // vertical exchanges
        for y in 0..INTERNAL_EDGES_LENGTH {
            let y = y as u32;
            for x in 0..CHUNK_SIZE {
                let x = x as u32;
                let from_pos = uvec2(x, y);
                let to_pos = uvec2(x, y + 1);

                let [mix_from, mix_to] = mixtures
                    .tile_view_many_mut([from_pos, to_pos])
                    .expect("pos valid");
                let [cached_from, cached_to] =
                    cached.get_many([from_pos, to_pos]).expect("pos valid");
                let exchanged_amounts =
                    exchange(&gas_list, mix_from, mix_to, cached_from, cached_to);
                let exchanged_sum = exchanged_amounts.reduce_add();

                let [flow_from, flow_to] =
                    flows.get_many_mut([from_pos, to_pos]).expect("pos valid");

                let flow_direction = Vec2::Y;
                let force = exchanged_sum * NEWTONS_PER_MOLE;
                *flow_from += flow_direction * force;
                *flow_to += flow_direction * force;
            }
        }
    }
}

fn apply_external_exchanges(
    gas_list: Res<GasList>,
    grid: Query<&Grid>,
    mut chunk_query: Query<(&mut ChunkMixtures, &mut Flows, &ChunkCached)>,
) {
    let grid = grid.single().expect("grid exists");

    let mut processed = HashSet::new();

    for (&position, &entity) in grid.chunks.iter() {
        let directions = [
            (IVec2::X, INTERNAL_EDGES_LENGTH as u32, 0),
            (IVec2::NEG_X, 0, INTERNAL_EDGES_LENGTH as u32),
            (IVec2::Y, INTERNAL_EDGES_LENGTH as u32, 0),
            (IVec2::NEG_Y, 0, INTERNAL_EDGES_LENGTH as u32),
        ];

        for (offset, fixed_axis_current, fixed_axis_neighbor) in directions {
            let neighbor_pos = position + offset;

            let Some(&neighbor_entity) = grid.chunks.get(&neighbor_pos) else {
                continue;
            };

            if processed.contains(&neighbor_entity) {
                continue;
            }

            if let Ok(
                [
                    (mut current_mixtures, mut current_flows, current_cached),
                    (mut neighbor_mixtures, mut neighbor_flows, neighbor_cached),
                ],
            ) = chunk_query.get_many_mut([entity, neighbor_entity])
            {
                let flow_dir = offset.as_vec2();

                let length = CHUNK_SIZE as u32;
                for i in 0..length {
                    let (from_pos, to_pos) = if offset.y == 0 {
                        (uvec2(fixed_axis_current, i), uvec2(fixed_axis_neighbor, i))
                    } else {
                        (uvec2(i, fixed_axis_current), uvec2(i, fixed_axis_neighbor))
                    };

                    let mix_from = current_mixtures.tile_view_mut(from_pos).expect("pos valid");
                    let mix_to = neighbor_mixtures.tile_view_mut(to_pos).expect("pos valid");

                    let cached_from = current_cached.get(from_pos).expect("pos valid");
                    let cached_to = neighbor_cached.get(to_pos).expect("pos valid");

                    let exchanged_amounts =
                        exchange(&gas_list, mix_from, mix_to, cached_from, cached_to);
                    let exchanged_sum = exchanged_amounts.reduce_add();

                    let flow_from = current_flows.get_mut(from_pos).expect("pos valid");
                    let flow_to = neighbor_flows.get_mut(to_pos).expect("pos valid");

                    let force = exchanged_sum * NEWTONS_PER_MOLE;
                    *flow_from += flow_dir * force;
                    *flow_to += flow_dir * force;
                }
            }
        }
        processed.insert(entity);
    }
}

fn cull_mixtures(active_chunks: Query<Mut<ChunkMixtures>>) {
    for mut chunk in active_chunks {
        let chunk = chunk.bypass_change_detection();
        chunk.cull();
    }
}

fn exchange(
    gas_list: &GasList,
    mut from: TileMixtureViewMut,
    mut dest: TileMixtureViewMut,
    cached_from: &CachedTile,
    cached_dest: &CachedTile,
) -> PerGasArray {
    let delta_partial_pressures = cached_from.partial_pressures - cached_dest.partial_pressures;
    let gas_type_flow_direction = delta_partial_pressures.simd_gt(f32x16::ZERO);

    let from_temp_vec = f32x16::splat(cached_from.temperature);
    let dest_temp_vec = f32x16::splat(cached_dest.temperature);
    let source_temp = gas_type_flow_direction.blend(from_temp_vec, dest_temp_vec);

    let volume = *from.volume(); // assuming `from` and `dest` share volume
    let moles_to_move = ideal_gas_law_moles_wide(delta_partial_pressures, volume, source_temp);

    let exchanging_amounts = moles_to_move * BASE_DIFFUSION_COEFFICIENT;

    let molar_heat_capacities = gas_list.get_molar_heat_capacities();
    let delta_energy_lanes = exchanging_amounts * molar_heat_capacities * source_temp;
    let exchanging_energy = delta_energy_lanes.reduce_add();

    // molar transfer
    *from.moles_mut() -= exchanging_amounts;
    *dest.moles_mut() += exchanging_amounts;

    // energy transfer
    *from.energy_mut() -= exchanging_energy;
    *dest.energy_mut() += exchanging_energy;

    exchanging_amounts
}

fn update_space_clear(mut chunks: Query<(Mut<ChunkMixtures>, &SpaceChunk)>) {
    for (mut chunk, space) in chunks.iter_mut() {
        let chunk = chunk.bypass_change_detection();
        space.iter_with_pos().for_each(|(pos, is_space)| {
            if *is_space {
                chunk.tile_view_mut(pos).expect("pos valid").clear();
            }
        });
    }
}

#[derive(QueryFilter)]
struct ChangingChunks {
    or_changed: Or<(
        Changed<ChunkMixtures>,
        Changed<SpaceChunk>,
        Changed<ImpermeableChunk>,
    )>,
}
