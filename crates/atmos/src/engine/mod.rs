pub mod active;
pub mod chunk;
pub(crate) mod schedule;

use bevy::{ecs::query::QueryFilter, prelude::*};

use grid::{CHUNK_SIZE, Chunk, Grid};
use uom::{
    ConstZero,
    si::{
        amount_of_substance::AmountOfSubstance, f32::*, pressure::kilopascal, volume::cubic_meter,
    },
};

use crate::{
    BASE_DIFFUSION_COEFFICIENT, MAX_NUMBER_OF_GASES, MINIMUM_DELTA_PRESSURE,
    NEWTONS_PER_KILOPASCAL,
    engine::{
        active::{Active, ProcessedTick},
        chunk::{
            ChunkDeltas, DELTAS_LENGTH, Delta, ImpermeableChunk, InterchunkDeltas, Mixtures,
            SpaceChunk,
        },
        schedule::{AtmosSchedule, run_atmos_schedule},
    },
    gas_mixture::ideal_gas_law_moles,
    iter_gas_ids,
    prelude::*,
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
    let zero_delta_pressures = [Pressure::ZERO; MAX_NUMBER_OF_GASES];

    for (mixtures, space, _impermeable, _) in active_chunks.iter() {
        let chunk = mixtures;
        space.iter_with_pos().for_each(|(pos, is_space)| {
            if *is_space {
                // # Safety
                //
                // position valid within chunk via iter_with_pos()
                unsafe {
                    let mix = chunk.mixtures.get_unchecked(pos);
                    if mix.energy != Energy::ZERO {
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
                let deltas = if impermeable {
                    zero_delta_pressures
                } else {
                    let (l, r) = mixtures.mixtures().get_two(lhs, rhs).expect("pos valid");
                    l.delta_pressures(r, &gas_list)
                };

                chunk_deltas.horizontals.set(lhs, deltas);
            }
        }

        // horizontal deltas
        for y in 0..DELTAS_LENGTH {
            let y = y as u32;
            for x in 0..CHUNK_SIZE {
                let x = x as u32;
                let lhs = uvec2(x, y);
                let rhs = uvec2(x, y + 1);

                let impermeable = *impermeable.get(lhs).expect("pos valid")
                    || *impermeable.get(rhs).expect("pos valid");
                let deltas = if impermeable {
                    zero_delta_pressures
                } else {
                    let (l, r) = mixtures.mixtures().get_two(lhs, rhs).expect("pos valid");
                    l.delta_pressures(r, &gas_list)
                };

                chunk_deltas.verticals.set(lhs, deltas);
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
                    if mix.energy != Energy::ZERO {
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

                let deltas = chunk_deltas.horizontals.get(lhs).expect("pos valid");
                let deltas_abs = deltas.map(|d| d.abs());
                let deltas_abs_sum: Pressure = deltas_abs.iter().copied().sum();
                if deltas_abs_sum < Pressure::new::<kilopascal>(MINIMUM_DELTA_PRESSURE) {
                    continue;
                }
                // dirty.0 = true;

                let mixtures = mixtures.as_mut();
                let (mix_lhs, mix_rhs) = mixtures
                    .mixtures_mut()
                    .get_two_mut(lhs, rhs)
                    .expect("pos valid");
                exchange_with_deltas(&gas_list, mix_lhs, mix_rhs, deltas);

                // perhaps flows can be applied parellel.
                let (flow_lhs, flow_rhs) = mixtures
                    .flows_mut()
                    .get_two_mut(lhs, rhs)
                    .expect("pos valid");
                let flow_direction = Vec2::X;
                let deltas_sum: Pressure = deltas.iter().copied().sum();
                let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                *flow_lhs += flow_direction * force;
                *flow_rhs += flow_direction * force;
            }
        }

        // horizontal deltas
        for y in 0..DELTAS_LENGTH {
            let y = y as u32;
            for x in 0..CHUNK_SIZE {
                let x = x as u32;
                let lhs = uvec2(x, y);
                let rhs = uvec2(x, y + 1);

                let deltas = chunk_deltas.verticals.get(lhs).expect("pos valid");
                let deltas_abs = deltas.map(|d| d.abs());
                let deltas_abs_sum: Pressure = deltas_abs.iter().copied().sum();
                if deltas_abs_sum < Pressure::new::<kilopascal>(MINIMUM_DELTA_PRESSURE) {
                    continue;
                }

                let mixtures = mixtures.as_mut();
                let (mix_lhs, mix_rhs) = mixtures
                    .mixtures_mut()
                    .get_two_mut(lhs, rhs)
                    .expect("pos valid");
                exchange_with_deltas(&gas_list, mix_lhs, mix_rhs, deltas);

                // perhaps flows can be applied parellel.
                let (flow_lhs, flow_rhs) = mixtures
                    .flows_mut()
                    .get_two_mut(lhs, rhs)
                    .expect("pos valid");
                let flow_direction = Vec2::Y;
                let deltas_sum: Pressure = deltas.iter().copied().sum();
                let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                *flow_lhs += flow_direction * force;
                *flow_rhs += flow_direction * force;
            }
        }
    }
}

fn build_external_deltas(
    gas_list: Res<GasList>,
    grid: Single<&Grid>,
    mut active_chunks: Query<
        (&Chunk, &Mixtures, &ImpermeableChunk, &mut ChunkDeltas),
        With<Active>,
    >,
    neighbors: Query<(&Mixtures, &ImpermeableChunk)>,
) {
    let zero_delta_pressures = [Pressure::ZERO; MAX_NUMBER_OF_GASES];

    let space_mixture = GasMixture::new_empty(Volume::new::<cubic_meter>(2.5));

    // you think this is bad? dont look at the next function.
    for (chunk, current_mixtures, current_impermeable, mut deltas) in &mut active_chunks {
        let current_position = chunk.position();

        let left_chunk_position = current_position + IVec2::NEG_X;
        let left_chunk_opt = grid.get(left_chunk_position);

        for y in 0..CHUNK_SIZE {
            let y_u32 = y as u32;
            let curr_pos = uvec2(0, y_u32);
            let curr_wall = *current_impermeable.get(curr_pos).expect("pos valid");

            if curr_wall {
                deltas.left[y] = zero_delta_pressures;
                continue;
            }

            let lhs_mixture = current_mixtures
                .mixtures()
                .get(curr_pos)
                .expect("pos valid");

            if let Some(left_chunk_entity) = left_chunk_opt {
                let (left_mixtures, left_impermeable) = neighbors
                    .get(left_chunk_entity)
                    .expect("chunk should have atmos");
                let left_pos = uvec2(CHUNK_SIZE as u32 - 1, y_u32);
                let left_wall = *left_impermeable.get(left_pos).expect("pos valid");

                if left_wall {
                    deltas.left[y] = zero_delta_pressures;
                } else {
                    let rhs_mixture = left_mixtures.mixtures().get(left_pos).expect("pos valid");
                    deltas.left[y] = lhs_mixture.delta_pressures(rhs_mixture, &gas_list);
                }
            } else {
                // if there is no chunk we just space it
                deltas.left[y] = lhs_mixture.delta_pressures(&space_mixture, &gas_list);
            }
        }

        let right_chunk_position = current_position + IVec2::X;
        let right_chunk_opt = grid.get(right_chunk_position);

        for y in 0..CHUNK_SIZE {
            let y_u32 = y as u32;
            let curr_pos = uvec2(CHUNK_SIZE as u32 - 1, y_u32);
            let curr_wall = *current_impermeable.get(curr_pos).expect("pos valid");

            if curr_wall {
                deltas.right[y] = zero_delta_pressures;
                continue;
            }

            let lhs_mixture = current_mixtures
                .mixtures()
                .get(curr_pos)
                .expect("pos valid");

            if let Some(right_chunk_entity) = right_chunk_opt {
                let (right_mixtures, right_impermeable) = neighbors
                    .get(right_chunk_entity)
                    .expect("chunk should have atmos");
                let right_pos = uvec2(0, y_u32);
                let right_wall = *right_impermeable.get(right_pos).expect("pos valid");

                if right_wall {
                    deltas.right[y] = zero_delta_pressures;
                } else {
                    let rhs_mixture = right_mixtures.mixtures().get(right_pos).expect("pos valid");
                    deltas.right[y] = lhs_mixture.delta_pressures(rhs_mixture, &gas_list);
                }
            } else {
                deltas.right[y] = lhs_mixture.delta_pressures(&space_mixture, &gas_list);
            }
        }

        // down
        let down_chunk_position = current_position + IVec2::Y;
        let down_chunk_opt = grid.get(down_chunk_position);

        for x in 0..CHUNK_SIZE {
            let x_u32 = x as u32;
            let curr_pos = uvec2(x_u32, CHUNK_SIZE as u32 - 1); // bot row iter
            let curr_wall = *current_impermeable.get(curr_pos).expect("pos valid");

            if curr_wall {
                deltas.down[x] = zero_delta_pressures;
                continue;
            }

            let lhs_mixture = current_mixtures
                .mixtures()
                .get(curr_pos)
                .expect("pos valid");

            if let Some(down_chunk_entity) = down_chunk_opt {
                let (down_mixtures, down_impermeable) = neighbors
                    .get(down_chunk_entity)
                    .expect("chunk should have atmos");
                let down_pos = uvec2(x_u32, 0);
                let down_wall = *down_impermeable.get(down_pos).expect("pos valid");

                if down_wall {
                    deltas.down[x] = zero_delta_pressures;
                } else {
                    let rhs_mixture = down_mixtures.mixtures().get(down_pos).expect("pos valid");
                    deltas.down[x] = lhs_mixture.delta_pressures(rhs_mixture, &gas_list);
                }
            } else {
                deltas.down[x] = lhs_mixture.delta_pressures(&space_mixture, &gas_list);
            }
        }

        // up
        let up_chunk_position = current_position + IVec2::NEG_Y;
        let up_chunk_opt = grid.get(up_chunk_position);

        for x in 0..CHUNK_SIZE {
            let x_u32 = x as u32;
            let curr_pos = uvec2(x_u32, 0); // top row iter
            let curr_wall = *current_impermeable.get(curr_pos).expect("pos valid");

            if curr_wall {
                deltas.up[x] = zero_delta_pressures;
                continue;
            }

            let lhs_mixture = current_mixtures
                .mixtures()
                .get(curr_pos)
                .expect("pos valid");

            if let Some(up_chunk_entity) = up_chunk_opt {
                let (up_mixtures, up_impermeable) = neighbors
                    .get(up_chunk_entity)
                    .expect("chunk should have atmos");
                let up_pos = uvec2(x_u32, CHUNK_SIZE as u32 - 1);
                let up_wall = *up_impermeable.get(up_pos).expect("pos valid");

                if up_wall {
                    deltas.up[x] = zero_delta_pressures;
                } else {
                    let rhs_mixture = up_mixtures.mixtures().get(up_pos).expect("pos valid");
                    deltas.up[x] = lhs_mixture.delta_pressures(rhs_mixture, &gas_list);
                }
            } else {
                deltas.up[x] = lhs_mixture.delta_pressures(&space_mixture, &gas_list);
            }
        }
    }
}

/// I am SORRY
/// This 180 line function iterates each edge of each active chunk
/// for each chunk it checks each direction and applies deltas onto iterates
/// if possible (chunk might not exist or its blocked and so on)
fn apply_external_deltas(
    gas_list: Res<GasList>,
    atmos_res: Res<AtmosphericsResource>,
    grid: Single<&Grid>,
    active_chunks: Query<(Entity, &Chunk, &ChunkDeltas), With<Active>>,
    mut processed_ticks: Query<&mut ProcessedTick>,
    mut chunks: Query<Mut<Mixtures>>,
) {
    let space_volume = Volume::new::<cubic_meter>(2.5);
    let current_tick = atmos_res.current_tick;

    let is_edge_active = |edge_deltas: &InterchunkDeltas| -> bool {
        edge_deltas.iter().any(|deltas| {
            let deltas_abs_sum: Pressure = deltas.map(|d| d.abs()).iter().copied().sum();
            deltas_abs_sum >= Pressure::new::<kilopascal>(MINIMUM_DELTA_PRESSURE)
        })
    };

    // shitcode begins NOW
    for (current_entity, chunk, deltas) in &active_chunks {
        if let Ok(mut pt) = processed_ticks.get_mut(current_entity) {
            pt.0 = current_tick;
        }

        let current_position = chunk.position();

        // left
        let left_chunk_position = current_position + IVec2::NEG_X;
        if is_edge_active(&deltas.left) {
            let mut skip = false;
            if let Some(left_chunk_entity) = grid.get(left_chunk_position)
                && let Ok(pt) = processed_ticks.get(left_chunk_entity)
                && pt.0 == current_tick
            {
                skip = true;
            }

            if !skip {
                if let Some(left_chunk_entity) = grid.get(left_chunk_position) {
                    if let Ok([mut current_mixtures, mut left_mixtures]) =
                        chunks.get_many_mut([current_entity, left_chunk_entity])
                    {
                        for y in 0..CHUNK_SIZE {
                            let y_u32 = y as u32;
                            let curr_pos = uvec2(0, y_u32);
                            let left_pos = uvec2(CHUNK_SIZE as u32 - 1, y_u32);

                            let lhs_mixture = current_mixtures
                                .mixtures_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let rhs_mixture = left_mixtures
                                .mixtures_mut()
                                .get_mut(left_pos)
                                .expect("pos valid");

                            exchange_with_deltas(
                                &gas_list,
                                lhs_mixture,
                                rhs_mixture,
                                &deltas.left[y],
                            );

                            let flow_lhs = current_mixtures
                                .flows_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let flow_rhs = left_mixtures
                                .flows_mut()
                                .get_mut(left_pos)
                                .expect("pos valid");
                            let deltas_sum: Pressure = deltas.left[y].iter().copied().sum();
                            let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                            *flow_lhs += Vec2::X * force;
                            *flow_rhs += Vec2::X * force;
                        }
                    }
                } else {
                    if let Ok(mut current_mixtures) = chunks.get_mut(current_entity) {
                        for y in 0..CHUNK_SIZE {
                            let y_u32 = y as u32;
                            let curr_pos = uvec2(0, y_u32);
                            let mut space_mixture = GasMixture::new_empty(space_volume);
                            let lhs_mixture = current_mixtures
                                .mixtures_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            exchange_with_deltas(
                                &gas_list,
                                lhs_mixture,
                                &mut space_mixture,
                                &deltas.left[y],
                            );

                            let flow_lhs = current_mixtures
                                .flows_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let deltas_sum: Pressure = deltas.left[y].iter().copied().sum();
                            let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                            *flow_lhs += Vec2::X * force;
                        }
                    }
                }
            }
        }

        // right
        let right_chunk_position = current_position + IVec2::X;
        if is_edge_active(&deltas.right) {
            let mut skip = false;
            if let Some(right_chunk_entity) = grid.get(right_chunk_position)
                && let Ok(pt) = processed_ticks.get(right_chunk_entity)
                && pt.0 == current_tick
            {
                skip = true;
            }

            if !skip {
                if let Some(right_chunk_entity) = grid.get(right_chunk_position) {
                    if let Ok([mut current_mixtures, mut right_mixtures]) =
                        chunks.get_many_mut([current_entity, right_chunk_entity])
                    {
                        for y in 0..CHUNK_SIZE {
                            let y_u32 = y as u32;
                            let curr_pos = uvec2(CHUNK_SIZE as u32 - 1, y_u32);
                            let right_pos = uvec2(0, y_u32);

                            let lhs_mixture = current_mixtures
                                .mixtures_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let rhs_mixture = right_mixtures
                                .mixtures_mut()
                                .get_mut(right_pos)
                                .expect("pos valid");

                            exchange_with_deltas(
                                &gas_list,
                                lhs_mixture,
                                rhs_mixture,
                                &deltas.right[y],
                            );

                            let flow_lhs = current_mixtures
                                .flows_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let flow_rhs = right_mixtures
                                .flows_mut()
                                .get_mut(right_pos)
                                .expect("pos valid");
                            let deltas_sum: Pressure = deltas.right[y].iter().copied().sum();
                            let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                            *flow_lhs += Vec2::X * force;
                            *flow_rhs += Vec2::X * force;
                        }
                    }
                } else {
                    if let Ok(mut current_mixtures) = chunks.get_mut(current_entity) {
                        for y in 0..CHUNK_SIZE {
                            let y_u32 = y as u32;
                            let curr_pos = uvec2(CHUNK_SIZE as u32 - 1, y_u32);
                            let mut space_mixture = GasMixture::new_empty(space_volume);
                            let lhs_mixture = current_mixtures
                                .mixtures_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            exchange_with_deltas(
                                &gas_list,
                                lhs_mixture,
                                &mut space_mixture,
                                &deltas.right[y],
                            );

                            let flow_lhs = current_mixtures
                                .flows_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let deltas_sum: Pressure = deltas.right[y].iter().copied().sum();
                            let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                            *flow_lhs += Vec2::X * force;
                        }
                    }
                }
            }
        }

        // down
        let down_chunk_position = current_position + IVec2::Y;
        if is_edge_active(&deltas.down) {
            let mut skip = false;
            if let Some(down_chunk_entity) = grid.get(down_chunk_position)
                && let Ok(pt) = processed_ticks.get(down_chunk_entity)
                && pt.0 == current_tick
            {
                skip = true;
            }

            if !skip {
                if let Some(down_chunk_entity) = grid.get(down_chunk_position) {
                    if let Ok([mut current_mixtures, mut down_mixtures]) =
                        chunks.get_many_mut([current_entity, down_chunk_entity])
                    {
                        for x in 0..CHUNK_SIZE {
                            let x_u32 = x as u32;
                            let curr_pos = uvec2(x_u32, CHUNK_SIZE as u32 - 1);
                            let down_pos = uvec2(x_u32, 0);

                            let lhs_mixture = current_mixtures
                                .mixtures_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let rhs_mixture = down_mixtures
                                .mixtures_mut()
                                .get_mut(down_pos)
                                .expect("pos valid");

                            exchange_with_deltas(
                                &gas_list,
                                lhs_mixture,
                                rhs_mixture,
                                &deltas.down[x],
                            );

                            let flow_lhs = current_mixtures
                                .flows_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let flow_rhs = down_mixtures
                                .flows_mut()
                                .get_mut(down_pos)
                                .expect("pos valid");
                            let deltas_sum: Pressure = deltas.down[x].iter().copied().sum();
                            let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                            *flow_lhs += Vec2::Y * force;
                            *flow_rhs += Vec2::Y * force;
                        }
                    }
                } else {
                    if let Ok(mut current_mixtures) = chunks.get_mut(current_entity) {
                        for x in 0..CHUNK_SIZE {
                            let x_u32 = x as u32;
                            let curr_pos = uvec2(x_u32, CHUNK_SIZE as u32 - 1);
                            let mut space_mixture = GasMixture::new_empty(space_volume);
                            let lhs_mixture = current_mixtures
                                .mixtures_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");

                            exchange_with_deltas(
                                &gas_list,
                                lhs_mixture,
                                &mut space_mixture,
                                &deltas.down[x],
                            );

                            let flow_lhs = current_mixtures
                                .flows_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let deltas_sum: Pressure = deltas.down[x].iter().copied().sum();
                            let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                            *flow_lhs += Vec2::Y * force;
                        }
                    }
                }
            }
        }

        // up
        let up_chunk_position = current_position + IVec2::NEG_Y;
        if is_edge_active(&deltas.up) {
            let mut skip = false;
            if let Some(up_chunk_entity) = grid.get(up_chunk_position)
                && let Ok(pt) = processed_ticks.get(up_chunk_entity)
                && pt.0 == current_tick
            {
                skip = true;
            }

            if !skip {
                if let Some(up_chunk_entity) = grid.get(up_chunk_position) {
                    if let Ok([mut current_mixtures, mut up_mixtures]) =
                        chunks.get_many_mut([current_entity, up_chunk_entity])
                    {
                        for x in 0..CHUNK_SIZE {
                            let x_u32 = x as u32;
                            let curr_pos = uvec2(x_u32, 0);
                            let up_pos = uvec2(x_u32, CHUNK_SIZE as u32 - 1);

                            let lhs_mixture = current_mixtures
                                .mixtures_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let rhs_mixture = up_mixtures
                                .mixtures_mut()
                                .get_mut(up_pos)
                                .expect("pos valid");

                            exchange_with_deltas(
                                &gas_list,
                                lhs_mixture,
                                rhs_mixture,
                                &deltas.up[x],
                            );

                            let flow_lhs = current_mixtures
                                .flows_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let flow_rhs =
                                up_mixtures.flows_mut().get_mut(up_pos).expect("pos valid");
                            let deltas_sum: Pressure = deltas.up[x].iter().copied().sum();
                            let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                            *flow_lhs += Vec2::Y * force;
                            *flow_rhs += Vec2::Y * force;
                        }
                    }
                } else {
                    if let Ok(mut current_mixtures) = chunks.get_mut(current_entity) {
                        for x in 0..CHUNK_SIZE {
                            let x_u32 = x as u32;
                            let curr_pos = uvec2(x_u32, 0);
                            let mut space_mixture = GasMixture::new_empty(space_volume);
                            let lhs_mixture = current_mixtures
                                .mixtures_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            exchange_with_deltas(
                                &gas_list,
                                lhs_mixture,
                                &mut space_mixture,
                                &deltas.up[x],
                            );

                            let flow_lhs = current_mixtures
                                .flows_mut()
                                .get_mut(curr_pos)
                                .expect("pos valid");
                            let deltas_sum: Pressure = deltas.up[x].iter().copied().sum();
                            let force = NEWTONS_PER_KILOPASCAL * deltas_sum.get::<kilopascal>();
                            *flow_lhs += Vec2::Y * force;
                        }
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
    lhs: &mut GasMixture,
    rhs: &mut GasMixture,
    deltas: &Delta,
) {
    let molar_heat_capacities = gas_list.get_molar_heat_capacities();
    let lhs_temp = lhs.temperature(gas_list);
    let rhs_temp = rhs.temperature(gas_list);

    let lhs_to_rhs = deltas.map(|dp| dp > Pressure::ZERO);
    let mut moved_moles = [AmountOfSubstance::ZERO; MAX_NUMBER_OF_GASES];

    for gas_id in iter_gas_ids() {
        let lhs_to_rhs = lhs_to_rhs[gas_id];
        let delta_pressure = deltas[gas_id];

        let source_temp = if lhs_to_rhs { lhs_temp } else { rhs_temp };

        let volume = lhs.volume(); // lhs & rhs volume should be the same

        // NOTE: denominator on this function could be shared.
        let moles_to_move = ideal_gas_law_moles(delta_pressure.abs(), volume, source_temp);
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

    let mut total_energy_transfer: Energy = Energy::ZERO;
    for gas_id in iter_gas_ids() {
        let moles_to_move = moved_moles[gas_id];
        let lhs_to_rhs = lhs_to_rhs[gas_id];

        let molar_heat_capacity = molar_heat_capacities[gas_id];
        let temperature = if lhs_to_rhs { lhs_temp } else { rhs_temp };

        let delta_energy = moles_to_move * molar_heat_capacity * temperature;

        total_energy_transfer += delta_energy;
    }

    lhs.energy -= total_energy_transfer;
    rhs.energy += total_energy_transfer;
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
                if mix.energy != Energy::ZERO {
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
