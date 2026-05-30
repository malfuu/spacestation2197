pub mod chunk;
pub mod excited;
pub(crate) mod schedule;

#[doc(hidden)]
pub mod prelude;
pub mod tile_mixture;

use bevy::{ecs::query::QueryFilter, prelude::*};

use tile_grid::{CHUNK_SIZE, Chunk, Grid, LocalTilePosition};

use atmos_primitives::{
    BASE_DIFFUSION_COEFFICIENT, MINIMUM_DELTA_PRESSURE, NEWTONS_PER_KILOPASCAL,
    gas_mixture::ideal_gas_law_moles, iter_gas_ids, prelude::*,
};

use crate::{
    chunk::{
        ChunkDeltas, ChunkMixtures, DELTAS_LENGTH, Delta, Flows, ImpermeableChunk,
        InterchunkDeltas, SpaceChunk,
    },
    excited::{Excited, ProcessedTick},
    schedule::{AtmosSchedule, run_atmos_schedule},
    tile_mixture::{TileMixtureView, TileMixtureViewMut},
};

const TICKS_TO_SLEEP: u32 = 1;

pub type AtmosTick = u32;

#[derive(Resource, Debug)]
pub struct AtmosphericsResource {
    pub enabled: bool,
    current_tick: AtmosTick,
}

pub struct AtmosphericsPlugin;

impl Plugin for AtmosphericsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AtmosphericsResource {
            enabled: true,
            current_tick: 0,
        })
        .init_schedule(AtmosSchedule)
        .configure_sets(FixedUpdate, AtmosSystems::AtmosTick)
        .add_systems(
            FixedUpdate,
            run_atmos_schedule.in_set(AtmosSystems::AtmosTick),
        )
        .add_systems(
            AtmosSchedule,
            (
                // active management
                wake_chunks,
                update_active_ticks,
                // actual simulation
                reset_flows,
                cull_mixtures,
                update_space_clear,
                build_internal_deltas,
                build_external_deltas,
                apply_internal_deltas,
                apply_external_deltas,
                // sleep if no internal changes
                sleep_chunks,
            )
                .chain(),
        );
    }
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AtmosSystems {
    /// Runs atmospherics simulation.
    AtmosTick,
}

fn reset_flows(active_chunks: Query<Mut<Flows>, With<Excited>>) {
    for mut flows in active_chunks {
        flows.iter_mut().for_each(|f| *f = Vec2::ZERO);
    }
}

fn build_internal_deltas(
    gas_list: Res<GasList>,
    active_chunks: Query<(&ChunkMixtures, &ImpermeableChunk, &mut ChunkDeltas), With<Excited>>,
) {
    let zero_delta_pressures_pa = per_gas_array(0.0);

    for (mixtures, impermeable, mut chunk_deltas) in active_chunks {
        let calculate_deltas = |lhs: LocalTilePosition, rhs: LocalTilePosition| {
            if *impermeable.get(lhs).unwrap() || *impermeable.get(rhs).unwrap() {
                return zero_delta_pressures_pa;
            }
            let [l, r] = mixtures.tile_view_many([lhs, rhs]).unwrap();
            l.delta_pressures(&r, &gas_list)
        };

        // horizontal deltas
        for y in 0..CHUNK_SIZE as u32 {
            for x in 0..DELTAS_LENGTH as u32 {
                let pos = uvec2(x, y);
                chunk_deltas
                    .horizontals
                    .set(pos, calculate_deltas(pos, pos + LocalTilePosition::X));
            }
        }

        // vertical deltas
        for y in 0..DELTAS_LENGTH as u32 {
            for x in 0..CHUNK_SIZE as u32 {
                let pos = uvec2(x, y);
                chunk_deltas
                    .verticals
                    .set(pos, calculate_deltas(pos, pos + LocalTilePosition::Y));
            }
        }
    }
}

type ActiveChunkData<'w> = (
    Mut<'w, ChunkMixtures>,
    Mut<'w, Flows>,
    &'static ChunkDeltas,
    &'static SpaceChunk,
);

fn apply_internal_deltas(
    gas_list: Res<GasList>,
    active_chunks: Query<ActiveChunkData, With<Excited>>,
) {
    for (mut mixtures, mut flows, chunk_deltas, _) in active_chunks {
        // horizontal deltas
        for y in 0..CHUNK_SIZE {
            let y = y as u32;
            for x in 0..DELTAS_LENGTH {
                let x = x as u32;
                let lhs = uvec2(x, y);
                let rhs = uvec2(x + 1, y);

                let deltas_pa = chunk_deltas.horizontals.get(lhs).expect("pos valid");
                let deltas_abs = deltas_pa.map(|d| d.abs());
                let deltas_abs_sum: f32 = deltas_abs.iter().copied().sum();

                if deltas_abs_sum < MINIMUM_DELTA_PRESSURE * 1000.0 {
                    continue;
                }
                // dirty.0 = true;

                let [mix_lhs, mix_rhs] =
                    mixtures.tile_view_many_mut([lhs, rhs]).expect("pos valid");
                exchange_with_deltas(&gas_list, mix_lhs, mix_rhs, deltas_pa);

                // perhaps flows can be applied parallel.
                let [flow_lhs, flow_rhs] = flows.get_many_mut([lhs, rhs]).expect("pos valid");
                let flow_direction = Vec2::X;
                let deltas_sum: f32 = deltas_pa.iter().copied().sum();

                let force = NEWTONS_PER_KILOPASCAL * (deltas_sum / 1000.0);
                *flow_lhs += flow_direction * force;
                *flow_rhs += flow_direction * force;
            }
        }

        // vertical deltas
        for y in 0..DELTAS_LENGTH {
            let y = y as u32;
            for x in 0..CHUNK_SIZE {
                let x = x as u32;
                let lhs = uvec2(x, y);
                let rhs = uvec2(x, y + 1);

                let deltas_pa = chunk_deltas.verticals.get(lhs).expect("pos valid");
                let deltas_abs = deltas_pa.map(|d| d.abs());
                let deltas_abs_sum: f32 = deltas_abs.iter().copied().sum();

                if deltas_abs_sum < MINIMUM_DELTA_PRESSURE * 1000.0 {
                    continue;
                }

                let [mix_lhs, mix_rhs] =
                    mixtures.tile_view_many_mut([lhs, rhs]).expect("pos valid");
                exchange_with_deltas(&gas_list, mix_lhs, mix_rhs, deltas_pa);

                // perhaps flows can be applied parallel.
                let [flow_lhs, flow_rhs] = flows.get_many_mut([lhs, rhs]).expect("pos valid");
                let flow_direction = Vec2::Y;
                let deltas_sum: f32 = deltas_pa.iter().copied().sum();

                let force = NEWTONS_PER_KILOPASCAL * (deltas_sum / 1000.0);
                *flow_lhs += flow_direction * force;
                *flow_rhs += flow_direction * force;
            }
        }
    }
}

type BuildExternalDeltaHelper<'a> = (
    IVec2,
    &'a mut InterchunkDeltas,
    fn(u32) -> LocalTilePosition,
    fn(u32) -> LocalTilePosition,
);

fn build_external_deltas(
    gas_list: Res<GasList>,
    grid: Single<&Grid>,
    mut active_chunks: Query<
        (&Chunk, &ChunkMixtures, &ImpermeableChunk, &mut ChunkDeltas),
        With<Excited>,
    >,
    neighbors: Query<(&ChunkMixtures, &ImpermeableChunk)>,
) {
    let space_moles = per_gas_array(0.0);
    let space_energy = 0.0;
    let space_mixture = TileMixtureView::new(&space_moles, &space_energy);

    let zero_delta_pressures_pa = per_gas_array(0.0);

    for (chunk, current_chunk_atmos, current_impermeable, deltas) in &mut active_chunks {
        let current_position = chunk.position();

        let deltas = deltas.into_inner();

        // chunk offset, delta array, curr_chunk tile mapper, neighbor chunk tile mapper
        let edges: [BuildExternalDeltaHelper; 4] = [
            (
                IVec2::NEG_X,
                &mut deltas.left,
                |i| uvec2(0, i),
                |i| uvec2(CHUNK_SIZE as u32 - 1, i),
            ),
            (
                IVec2::X,
                &mut deltas.right,
                |i| uvec2(CHUNK_SIZE as u32 - 1, i),
                |i| uvec2(0, i),
            ),
            (
                IVec2::Y,
                &mut deltas.down,
                |i| uvec2(i, CHUNK_SIZE as u32 - 1),
                |i| uvec2(i, 0),
            ),
            (
                IVec2::NEG_Y,
                &mut deltas.up,
                |i| uvec2(i, 0),
                |i| uvec2(i, CHUNK_SIZE as u32 - 1),
            ),
        ];

        for (chunk_offset, delta_array, get_curr_pos, get_neighbor_pos) in edges {
            let neighbor_chunk_opt = grid.get(current_position + chunk_offset);

            for (i, delta_array_i) in delta_array.iter_mut().enumerate().take(CHUNK_SIZE) {
                let i_u32 = i as u32;
                let curr_pos = get_curr_pos(i_u32);
                let curr_wall = *current_impermeable.get(curr_pos).expect("pos valid");

                if curr_wall {
                    *delta_array_i = zero_delta_pressures_pa;
                    continue;
                }

                let lhs_mixture = current_chunk_atmos.tile_view(curr_pos).expect("pos valid");

                if let Some(neighbor_entity) = neighbor_chunk_opt {
                    let (neighbor_atmos, neighbor_impermeable) = neighbors
                        .get(neighbor_entity)
                        .expect("chunk should have atmos");

                    let neighbor_pos = get_neighbor_pos(i_u32);
                    let neighbor_wall = *neighbor_impermeable.get(neighbor_pos).expect("pos valid");

                    if neighbor_wall {
                        *delta_array_i = zero_delta_pressures_pa;
                    } else {
                        let rhs_mixture =
                            neighbor_atmos.tile_view(neighbor_pos).expect("pos valid");

                        *delta_array_i = lhs_mixture.delta_pressures(&rhs_mixture, &gas_list);
                    }
                } else {
                    // space it if the chunk doesn't exist
                    *delta_array_i = lhs_mixture.delta_pressures(&space_mixture, &gas_list);
                }
            }
        }
    }
}

type ApplyExternalDeltaHelper<'a> = (
    IVec2,
    &'a InterchunkDeltas,
    fn(u32) -> LocalTilePosition,
    fn(u32) -> LocalTilePosition,
    Vec2,
);

fn apply_external_deltas(
    gas_list: Res<GasList>,
    atmos_res: Res<AtmosphericsResource>,
    grid: Single<&Grid>,
    active_chunks: Query<(Entity, &Chunk, &ChunkDeltas), With<Excited>>,
    mut processed_ticks: Query<&mut ProcessedTick>,
    mut chunks: Query<(Mut<ChunkMixtures>, Mut<Flows>)>,
) {
    let current_tick = atmos_res.current_tick;

    let is_edge_active = |edge_deltas: &InterchunkDeltas| -> bool {
        edge_deltas.iter().any(|deltas_pa| {
            let deltas_abs_sum: f32 = deltas_pa.map(|d| d.abs()).iter().copied().sum();
            deltas_abs_sum >= MINIMUM_DELTA_PRESSURE * 1000.0
        })
    };

    for (current_entity, chunk, deltas) in &active_chunks {
        if let Ok(mut pt) = processed_ticks.get_mut(current_entity) {
            pt.0 = current_tick;
        }

        let current_position = chunk.position();

        // chunk offset, delta array, curr_chunk tile mapper, neighbor chunk tile mapper, flow direction
        let edges: [ApplyExternalDeltaHelper; 4] = [
            (
                IVec2::NEG_X,
                &deltas.left,
                |i| uvec2(0, i),
                |i| uvec2(CHUNK_SIZE as u32 - 1, i),
                Vec2::X,
            ),
            (
                IVec2::X,
                &deltas.right,
                |i| uvec2(CHUNK_SIZE as u32 - 1, i),
                |i| uvec2(0, i),
                Vec2::X,
            ),
            (
                IVec2::Y,
                &deltas.down,
                |i| uvec2(i, CHUNK_SIZE as u32 - 1),
                |i| uvec2(i, 0),
                Vec2::Y,
            ),
            (
                IVec2::NEG_Y,
                &deltas.up,
                |i| uvec2(i, 0),
                |i| uvec2(i, CHUNK_SIZE as u32 - 1),
                Vec2::Y,
            ),
        ];

        for (chunk_offset, edge_deltas, get_curr_pos, get_neighbor_pos, flow_dir) in edges {
            if !is_edge_active(edge_deltas) {
                continue;
            }

            let neighbor_chunk_position = current_position + chunk_offset;
            let neighbor_chunk_opt = grid.get(neighbor_chunk_position);

            // skip if the neighbor has already processed this tick
            let skip = neighbor_chunk_opt
                .and_then(|entity| processed_ticks.get(entity).ok())
                .is_some_and(|pt| pt.0 == current_tick);

            if skip {
                continue;
            }

            if let Some(neighbor_entity) = neighbor_chunk_opt {
                // neighbor exists in this case
                if let Ok(
                    [
                        (mut current_mixtures, mut current_flows),
                        (mut neighbor_mixtures, mut neighbor_flows),
                    ],
                ) = chunks.get_many_mut([current_entity, neighbor_entity])
                {
                    for (i, _) in edge_deltas.iter().enumerate().take(CHUNK_SIZE) {
                        let i_u32 = i as u32;
                        let curr_pos = get_curr_pos(i_u32);
                        let neighbor_pos = get_neighbor_pos(i_u32);

                        let lhs_mixture =
                            current_mixtures.tile_view_mut(curr_pos).expect("pos valid");
                        let rhs_mixture = neighbor_mixtures
                            .tile_view_mut(neighbor_pos)
                            .expect("pos valid");

                        exchange_with_deltas(&gas_list, lhs_mixture, rhs_mixture, &edge_deltas[i]);

                        let flow_lhs = current_flows.get_mut(curr_pos).expect("pos valid");
                        let flow_rhs = neighbor_flows.get_mut(neighbor_pos).expect("pos valid");

                        let deltas_sum: f32 = edge_deltas[i].iter().copied().sum();
                        let force = NEWTONS_PER_KILOPASCAL * (deltas_sum / 1000.0);

                        *flow_lhs += flow_dir * force;
                        *flow_rhs += flow_dir * force;
                    }
                }
            } else {
                // neighbor does not exist in this case
                if let Ok((mut current_mixtures, mut current_flows)) =
                    chunks.get_mut(current_entity)
                {
                    for (i, _) in edge_deltas.iter().enumerate().take(CHUNK_SIZE) {
                        let i_u32 = i as u32;
                        let curr_pos = get_curr_pos(i_u32);

                        let mut space_moles = per_gas_array(0.0);
                        let mut space_energy = 0.0;
                        let space_mixture_view =
                            TileMixtureViewMut::new(&mut space_moles, &mut space_energy);

                        let lhs_mixture =
                            current_mixtures.tile_view_mut(curr_pos).expect("pos valid");

                        exchange_with_deltas(
                            &gas_list,
                            lhs_mixture,
                            space_mixture_view,
                            &edge_deltas[i],
                        );

                        let flow_lhs = current_flows.get_mut(curr_pos).expect("pos valid");

                        let deltas_sum: f32 = edge_deltas[i].iter().copied().sum();
                        let force = NEWTONS_PER_KILOPASCAL * (deltas_sum / 1000.0);

                        *flow_lhs += flow_dir * force;
                    }
                }
            }
        }
    }
}

fn cull_mixtures(active_chunks: Query<Mut<ChunkMixtures>, With<Excited>>) {
    for mut chunk in active_chunks {
        let chunk = chunk.bypass_change_detection();
        chunk.cull();
    }
}

fn exchange_with_deltas(
    gas_list: &GasList,
    mut lhs: TileMixtureViewMut,
    mut rhs: TileMixtureViewMut,
    deltas_pa: &Delta,
) {
    let molar_heat_capacities = gas_list.get_molar_heat_capacities();
    let lhs_temp_k = lhs.temperature(molar_heat_capacities);
    let rhs_temp_k = rhs.temperature(molar_heat_capacities);

    let lhs_to_rhs = deltas_pa.map(|dp| dp > 0.0);
    let mut moved_moles = per_gas_array(0.0);

    for gas_id in iter_gas_ids() {
        let lhs_to_rhs = lhs_to_rhs[gas_id];
        let delta_pressure_pa = deltas_pa[gas_id];

        let source_temp_k = if lhs_to_rhs { lhs_temp_k } else { rhs_temp_k };

        let volume_m3 = *lhs.volume(); // lhs & rhs volume should be the same

        // NOTE: denominator on this function could be shared.
        let moles_to_move = ideal_gas_law_moles(delta_pressure_pa.abs(), volume_m3, source_temp_k);
        let amount = moles_to_move * BASE_DIFFUSION_COEFFICIENT;

        moved_moles[gas_id] = if lhs_to_rhs {
            amount.min(lhs.moles()[gas_id])
        } else {
            -amount.min(rhs.moles()[gas_id])
        };
    }

    for gas_id in iter_gas_ids() {
        let n = moved_moles[gas_id];
        lhs.moles_mut()[gas_id] -= n;
        rhs.moles_mut()[gas_id] += n;
    }

    let mut total_energy_transfer_j: f32 = 0.0;
    for gas_id in iter_gas_ids() {
        let moles_to_move = moved_moles[gas_id];
        let lhs_to_rhs = lhs_to_rhs[gas_id];

        let molar_heat_capacity = molar_heat_capacities[gas_id];
        let temperature_k = if lhs_to_rhs { lhs_temp_k } else { rhs_temp_k };

        let delta_energy_j = moles_to_move * molar_heat_capacity * temperature_k;

        total_energy_transfer_j += delta_energy_j;
    }

    *lhs.energy_mut() -= total_energy_transfer_j;
    *rhs.energy_mut() += total_energy_transfer_j;
}

fn update_space_clear(mut chunks: Query<(Mut<ChunkMixtures>, &SpaceChunk), With<Excited>>) {
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
    no_actives: Without<Excited>,
    or_changed: Or<(
        Changed<ChunkMixtures>,
        Changed<SpaceChunk>,
        Changed<ImpermeableChunk>,
    )>,
}

/// Wakes up any inactive chunk that had its mixtures altered (e.g., by an external command).
fn wake_chunks(
    mut commands: Commands,
    resource: Res<AtmosphericsResource>,
    query: Query<Entity, ChangingChunks>,
) {
    for entity in &query {
        commands.entity(entity).insert(Excited {
            last_active_tick: resource.current_tick,
        });
    }
}

/// Refreshes the sleep timer for active chunks that had gas movement this tick.
fn update_active_ticks(
    resource: Res<AtmosphericsResource>,
    mut query: Query<&mut Excited, Changed<ChunkMixtures>>,
) {
    for mut active in &mut query {
        active.last_active_tick = resource.current_tick;
    }
}

/// Retires chunks that havent seen meaningful gas movement after TICKS_TO_SLEEP.
fn sleep_chunks(
    mut commands: Commands,
    resource: Res<AtmosphericsResource>,
    query: Query<(Entity, &Excited)>,
) {
    let current_tick = resource.current_tick;
    for (entity, active) in &query {
        if current_tick.saturating_sub(active.last_active_tick) > TICKS_TO_SLEEP {
            commands.entity(entity).remove::<Excited>();
        }
    }
}
