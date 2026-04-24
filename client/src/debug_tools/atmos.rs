use bevy::prelude::*;

use atmos_simulation::prelude::*;
use tile_grid::{Chunk, chunk_and_local_to_world};

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
                draw_wind_vectors.run_if(option_enabled(DEBUG_OPTION_ATMOS_WIND)),
            );
    }
}

fn ui_mixture_information() {}

fn draw_wind_vectors(
    mut gizmos: Gizmos<DebugGizmos>,
    chunks: Query<(&Chunk, &Flows), With<Excited>>,
) {
    for (chunk, flows) in chunks.iter() {
        let chunk_position = chunk.position();

        for (tile_position, flow) in flows.iter_with_pos() {
            let flow_len = flow.length();
            if flow_len <= f32::EPSILON {
                continue;
            }

            let global_position = chunk_and_local_to_world(chunk_position, tile_position).as_vec2();
            let start = Vec3::new(global_position.x + 0.5, 0.01, global_position.y + 0.5);

            let display_len = (flow_len / 32.0).tanh();

            let flow_dir = flow.normalize();
            let flow_offset = Vec3::new(flow_dir.x, 0.0, flow_dir.y) * display_len;
            let end = start + flow_offset;

            let intensity = display_len;
            let color = Color::srgb(intensity, 1.0 - intensity, 0.0);

            gizmos.arrow(start, end, color);
        }
    }
}
