use bevy::prelude::*;
use bevy_egui::prelude::*;

use atmos_primitives::gas_list::GasList;
use atmos_primitives::prelude::*;
use atmos_simulation::prelude::*;
use tile_grid::{Chunk, Grid, chunk_and_local_to_world, world_to_chunk_and_local};

use crate::base::input::ExtraInputs;
use crate::debug_tools::{AppDebugOptionExt, DebugGizmos, option_enabled};

const DEBUG_OPTION_ATMOS_MIXTURE: &str = "atmos_mixture";
const DEBUG_OPTION_ATMOS_WIND: &str = "atmos_wind";

pub(super) struct DebugAtmosPlugin;

impl Plugin for DebugAtmosPlugin {
    fn build(&self, app: &mut App) {
        app.register_debug_option(DEBUG_OPTION_ATMOS_MIXTURE)
            .register_debug_option(DEBUG_OPTION_ATMOS_WIND)
            .add_systems(
                Update,
                (
                    draw_wind_vectors.run_if(option_enabled(DEBUG_OPTION_ATMOS_WIND)),
                    ui_mixture_information.run_if(option_enabled(DEBUG_OPTION_ATMOS_MIXTURE)),
                ),
            );
    }
}

fn ui_mixture_information(
    mut contexts: EguiContexts,
    inputs: Res<ExtraInputs>,
    gas_list: Res<GasList>,
    grids: Query<&Grid>,
    chunk_mixtures: Query<&ChunkMixtures>,
) -> Result {
    let Some(mouse_positions) = inputs.mouse_positions() else {
        return Ok(());
    };

    let (chunk_pos, local_pos) = world_to_chunk_and_local(mouse_positions.tile_position);

    let mut found_mixture = None;
    let grid = grids.single().expect("only one grid");
    if let Some(chunk_entity) = grid.get(chunk_pos)
        && let Ok(mixtures) = chunk_mixtures.get(chunk_entity)
        && let Some(view) = mixtures.tile_view(local_pos)
    {
        found_mixture = Some(view);
    }

    egui::Window::new("Atmos Mixture").show(contexts.ctx_mut()?, |ui| {
        ui.label(format!("Chunk: {}, Local: {}", chunk_pos, local_pos));
        let Some(view) = found_mixture else {
            ui.label("No mixture found.");
            return;
        };

        let energy = view.energy();
        let temp = view.temperature(gas_list.get_molar_heat_capacities());
        let pressure = view.pressure(&gas_list);
        let total_moles = view.total_moles();

        ui.label(format!("Moles: {:.2}", total_moles));
        ui.label(format!("Energy: {:.2} J", energy));
        ui.label(format!("Temperature: {:.2} K", temp));
        ui.label(format!("Pressure: {:.2} kPa", pressure / 1000.0));

        ui.separator();

        let partial_pressures = view.partial_pressures(&gas_list);
        let moles = view.moles().as_array();

        ui.heading("Gases");
        for (i, gas) in gas_list.iter().enumerate() {
            let m = moles[i];
            if m > 0.0 {
                let partial_pressure = partial_pressures.as_array()[i];
                ui.label(format!(
                    "{}: {:.3} moles, {:.2} kPa",
                    gas.name,
                    m,
                    partial_pressure / 1000.0
                ));
            }
        }
    });

    Ok(())
}

fn draw_wind_vectors(mut gizmos: Gizmos<DebugGizmos>, chunks: Query<(&Chunk, &Flows)>) {
    for (chunk, flows) in chunks.iter() {
        let chunk_position = chunk.position();

        for (tile_position, flow) in flows.iter_with_pos() {
            let flow_len = flow.length();
            if flow_len <= f32::EPSILON {
                continue;
            }

            let global_position = chunk_and_local_to_world(chunk_position, tile_position).as_vec2();
            let start = Vec3::new(global_position.x + 0.5, 0.01, global_position.y + 0.5);

            let display_len = (flow_len / 64.0).tanh();

            let flow_dir = flow.normalize();
            let flow_offset = Vec3::new(flow_dir.x, 0.0, flow_dir.y) * display_len;
            let end = start + flow_offset;

            let intensity = display_len;
            let color = Color::srgb(intensity, 1.0 - intensity, 0.0);

            gizmos.arrow(start, end, color);
        }
    }
}
