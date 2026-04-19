pub mod active;
pub mod chunk;
pub(crate) mod schedule;

#[doc(hidden)]
pub mod prelude;

use bevy::{ecs::query::QueryFilter, prelude::*};

use grid::{CHUNK_SIZE, Chunk, Grid};

use atmos_primitives::{
    BASE_DIFFUSION_COEFFICIENT, MAX_NUMBER_OF_GASES, MINIMUM_DELTA_PRESSURE,
    NEWTONS_PER_KILOPASCAL, gas_mixture::ideal_gas_law_moles, iter_gas_ids, prelude::*,
};

use crate::{
    active::{Active, ProcessedTick},
    chunk::{
        ChunkDeltas, DELTAS_LENGTH, Delta, ImpermeableChunk, InterchunkDeltas, Mixtures, SpaceChunk,
    },
    schedule::{AtmosSchedule, run_atmos_schedule},
};

// I want to first apologize for the massive amounts of repeated code,
// particularly in building the chunk edge deltas.
// I give you permission to honorobly execute me after offending your eyes.
// TODO: fix this before more people commit seppuku.

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

fn reset_flows(active_chunks: Query<Mut<Mixtures>, With<Active>>) {
    for mut chunk in active_chunks {
        chunk
            .bypass_change_detection()
            .flows_mut()
            .iter_mut()
            .for_each(|f| *f = Vec2::ZERO);
    }
}

fn build_internal_deltas(
    gas_list: Res<GasList>,
    active_chunks: Query<
        (&Mixtures, &SpaceChunk, &ImpermeableChunk, &mut ChunkDeltas),
        With<Active>,
    >,
) {
    let zero_delta_pressures_pa = per_gas_array(0.0);

    for (mixtures, space, _impermeable, _) in active_chunks.iter() {
        let chunk = mixtures;
        space.iter_with_pos().for_each(|(pos, is_space)| {
            if *is_space {
                // # Safety
                //
                // position valid within chunk via iter_with_pos()
                unsafe {
                    let mix = chunk.mixtures.get_unchecked(pos);
                    if mix.energy != 0.0 {
                        panic!();
                    }
                }
            }
        });
    }

    for (mixtures, _space, impermeable, mut chunk_deltas) in active_chunks {
        // horizontal deltas
        for y in 0..CHUNK_SIZE {
            let y = y as u32;
            for x in 0..DELTAS_LENGTH {
                let x = x as u32;
                let lhs = uvec2(x, y);
                let rhs = uvec2(x + 1, y);

                let impermeable = *impermeable.get(lhs).expect("pos valid")
                    || *impermeable.get(rhs).expect("pos valid");
                let deltas_pa = if impermeable {
                    zero_delta_pressures_pa
                } else {
                    let (l, r) = mixtures.mixtures().get_two(lhs, rhs).expect("pos valid");
                    l.delta_pressures(r, &gas_list)
                };

                chunk_deltas.horizontals.set(lhs, deltas_pa);
            }
        }

        // vertical deltas
        for y in 0..DELTAS_LENGTH {
            let y = y as u32;
            for x in 0..CHUNK_SIZE {
                let x = x as u32;
                let lhs = uvec2(x, y);
                let rhs = uvec2(x, y + 1);

                let impermeable = *impermeable.get(lhs).expect("pos valid")
                    || *impermeable.get(rhs).expect("pos valid");
                let deltas_pa = if impermeable {
                    zero_delta_pressures_pa
                } else {
                    let (l, r) = mixtures.mixtures().get_two(lhs, rhs).expect("pos valid");
                    l.delta_pressures(r, &gas_list)
                };

                chunk_deltas.verticals.set(lhs, deltas_pa);
            }
        }
    }
}

fn apply_internal_deltas(
    gas_list: Res<GasList>,
    active_chunks: Query<(Mut<Mixtures>, &ChunkDeltas, &SpaceChunk), With<Active>>,
) {
    for (mixtures, _, space) in active_chunks.iter() {
        let chunk = mixtures;
        space.iter_with_pos().for_each(|(pos, is_space)| {
            if *is_space {
                // # Safety
                //
                // position valid within chunk via iter_with_pos()
                unsafe {
                    let mix = chunk.mixtures.get_unchecked(pos);
                    if mix.energy != 0.0 {
                        panic!();
                    }
                }
            }
        });
    }

    for (mut mixtures, chunk_deltas, _) in active_chunks {
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

                // MINIMUM_DELTA_PRESSURE assumes kPa, multiply by 1000 for Pascals
                if deltas_abs_sum < MINIMUM_DELTA_PRESSURE * 1000.0 {
                    continue;
                }
                // dirty.0 = true;

                let mixtures = mixtures.as_mut();
                let (mix_lhs, mix_rhs) = mixtures
                    .mixtures_mut()
                    .get_two_mut(lhs, rhs)
                    .expect("pos valid");
                exchange_with_deltas(&gas_list, mix_lhs, mix_rhs, deltas_pa);

                // perhaps flows can be applied parallel.
                let (flow_lhs, flow_rhs) = mixtures
                    .flows_mut()
                    .get_two_mut(lhs, rhs)
                    .expect("pos valid");
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

                let mixtures = mixtures.as_mut();
                let (mix_lhs, mix_rhs) = mixtures
                    .mixtures_mut()
                    .get_two_mut(lhs, rhs)
                    .expect("pos valid");
                exchange_with_deltas(&gas_list, mix_lhs, mix_rhs, deltas_pa);

                // perhaps flows can be applied parallel.
                let (flow_lhs, flow_rhs) = mixtures
                    .flows_mut()
                    .get_two_mut(lhs, rhs)
                    .expect("pos valid");
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
    fn(u32) -> UVec2,
    fn(u32) -> UVec2,
);

fn build_external_deltas(
    gas_list: Res<GasList>,
    grid: Single<&Grid>,
    mut active_chunks: Query<
        (&Chunk, &Mixtures, &ImpermeableChunk, &mut ChunkDeltas),
        With<Active>,
    >,
    neighbors: Query<(&Mixtures, &ImpermeableChunk)>,
) {
    let zero_delta_pressures_pa = [0.0; MAX_NUMBER_OF_GASES];
    let space_mixture = BasicGasMixture::new_empty(2.5);

    for (chunk, current_mixtures, current_impermeable, deltas) in &mut active_chunks {
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

                let lhs_mixture = current_mixtures
                    .mixtures()
                    .get(curr_pos)
                    .expect("pos valid");

                if let Some(neighbor_entity) = neighbor_chunk_opt {
                    let (neighbor_mixtures, neighbor_impermeable) = neighbors
                        .get(neighbor_entity)
                        .expect("chunk should have atmos");

                    let neighbor_pos = get_neighbor_pos(i_u32);
                    let neighbor_wall = *neighbor_impermeable.get(neighbor_pos).expect("pos valid");

                    if neighbor_wall {
                        *delta_array_i = zero_delta_pressures_pa;
                    } else {
                        let rhs_mixture = neighbor_mixtures
                            .mixtures()
                            .get(neighbor_pos)
                            .expect("pos valid");
                        *delta_array_i = lhs_mixture.delta_pressures(rhs_mixture, &gas_list);
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
    fn(u32) -> UVec2,
    fn(u32) -> UVec2,
    Vec2,
);

fn apply_external_deltas(
    gas_list: Res<GasList>,
    atmos_res: Res<AtmosphericsResource>,
    grid: Single<&Grid>,
    active_chunks: Query<(Entity, &Chunk, &ChunkDeltas), With<Active>>,
    mut processed_ticks: Query<&mut ProcessedTick>,
    mut chunks: Query<Mut<Mixtures>>,
) {
    let space_volume_m3 = 2.5;
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
                if let Ok([mut current_mixtures, mut neighbor_mixtures]) =
                    chunks.get_many_mut([current_entity, neighbor_entity])
                {
                    for (i, _) in edge_deltas.iter().enumerate().take(CHUNK_SIZE) {
                        let i_u32 = i as u32;
                        let curr_pos = get_curr_pos(i_u32);
                        let neighbor_pos = get_neighbor_pos(i_u32);

                        let lhs_mixture = current_mixtures
                            .mixtures_mut()
                            .get_mut(curr_pos)
                            .expect("pos valid");
                        let rhs_mixture = neighbor_mixtures
                            .mixtures_mut()
                            .get_mut(neighbor_pos)
                            .expect("pos valid");

                        exchange_with_deltas(&gas_list, lhs_mixture, rhs_mixture, &edge_deltas[i]);

                        let flow_lhs = current_mixtures
                            .flows_mut()
                            .get_mut(curr_pos)
                            .expect("pos valid");
                        let flow_rhs = neighbor_mixtures
                            .flows_mut()
                            .get_mut(neighbor_pos)
                            .expect("pos valid");

                        let deltas_sum: f32 = edge_deltas[i].iter().copied().sum();
                        let force = NEWTONS_PER_KILOPASCAL * (deltas_sum / 1000.0);

                        *flow_lhs += flow_dir * force;
                        *flow_rhs += flow_dir * force;
                    }
                }
            } else {
                // neighbor does not exist in this case
                if let Ok(mut current_mixtures) = chunks.get_mut(current_entity) {
                    for (i, _) in edge_deltas.iter().enumerate().take(CHUNK_SIZE) {
                        let i_u32 = i as u32;
                        let curr_pos = get_curr_pos(i_u32);
                        let mut space_mixture = BasicGasMixture::new_empty(space_volume_m3);

                        let lhs_mixture = current_mixtures
                            .mixtures_mut()
                            .get_mut(curr_pos)
                            .expect("pos valid");

                        exchange_with_deltas(
                            &gas_list,
                            lhs_mixture,
                            &mut space_mixture,
                            &edge_deltas[i],
                        );

                        let flow_lhs = current_mixtures
                            .flows_mut()
                            .get_mut(curr_pos)
                            .expect("pos valid");

                        let deltas_sum: f32 = edge_deltas[i].iter().copied().sum();
                        let force = NEWTONS_PER_KILOPASCAL * (deltas_sum / 1000.0);

                        *flow_lhs += flow_dir * force;
                    }
                }
            }
        }
    }
}

fn cull_mixtures(active_chunks: Query<Mut<Mixtures>, With<Active>>) {
    for mut chunk in active_chunks {
        // if !dirty.0 {
        //     continue;
        // }
        let chunk = chunk.bypass_change_detection();
        chunk.mixtures_mut().iter_mut().for_each(|m| m.cull());
    }
}

fn exchange_with_deltas(
    gas_list: &GasList,
    lhs: &mut BasicGasMixture,
    rhs: &mut BasicGasMixture,
    deltas_pa: &Delta,
) {
    let molar_heat_capacities = gas_list.get_molar_heat_capacities();
    let lhs_temp_k = lhs.temperature(gas_list);
    let rhs_temp_k = rhs.temperature(gas_list);

    let lhs_to_rhs = deltas_pa.map(|dp| dp > 0.0);
    let mut moved_moles = per_gas_array(0.0);

    for gas_id in iter_gas_ids() {
        let lhs_to_rhs = lhs_to_rhs[gas_id];
        let delta_pressure_pa = deltas_pa[gas_id];

        let source_temp_k = if lhs_to_rhs { lhs_temp_k } else { rhs_temp_k };

        let volume_m3 = lhs.volume(); // lhs & rhs volume should be the same

        // NOTE: denominator on this function could be shared.
        let moles_to_move = ideal_gas_law_moles(delta_pressure_pa.abs(), volume_m3, source_temp_k);
        let amount = moles_to_move * BASE_DIFFUSION_COEFFICIENT;

        moved_moles[gas_id] = if lhs_to_rhs {
            amount.min(lhs.contents[gas_id])
        } else {
            -amount.min(rhs.contents[gas_id])
        };
    }

    for gas_id in iter_gas_ids() {
        let n = moved_moles[gas_id];
        lhs.contents[gas_id] -= n;
        rhs.contents[gas_id] += n;
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

    lhs.energy -= total_energy_transfer_j;
    rhs.energy += total_energy_transfer_j;
}

fn update_space_clear(mut chunks: Query<(Mut<Mixtures>, &SpaceChunk), With<Active>>) {
    for (mut chunk, space) in chunks.iter_mut() {
        let chunk = chunk.bypass_change_detection();
        space.iter_with_pos().for_each(|(pos, is_space)| {
            if *is_space {
                // # Safety
                //
                // position valid within chunk via iter_with_pos()
                unsafe {
                    chunk.mixtures.get_unchecked_mut(pos).clear();
                }
            }
        });

        space.iter_with_pos().for_each(|(pos, is_space)| {
            if *is_space {
                // # Safety
                //
                // position valid within chunk via iter_with_pos()
                let mix = unsafe { chunk.mixtures.get_unchecked(pos) };
                if mix.energy != 0.0 {
                    panic!();
                }
            }
        });
    }
}

#[derive(QueryFilter)]
struct ChangingChunks {
    no_actives: Without<Active>,
    or_changed: Or<(
        Changed<Mixtures>,
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
        commands.entity(entity).insert(Active {
            last_active_tick: resource.current_tick,
        });
    }
}

/// Refreshes the sleep timer for active chunks that had gas movement this tick.
fn update_active_ticks(
    resource: Res<AtmosphericsResource>,
    mut query: Query<&mut Active, Changed<Mixtures>>,
) {
    for mut active in &mut query {
        active.last_active_tick = resource.current_tick;
    }
}

/// Retires chunks that havent seen meaningful gas movement after TICKS_TO_SLEEP.
fn sleep_chunks(
    mut commands: Commands,
    resource: Res<AtmosphericsResource>,
    query: Query<(Entity, &Active)>,
) {
    let current_tick = resource.current_tick;
    for (entity, active) in &query {
        if current_tick.saturating_sub(active.last_active_tick) > TICKS_TO_SLEEP {
            commands.entity(entity).remove::<Active>();
        }
    }
}
