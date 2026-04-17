use bevy::prelude::*;

use grid::{CHUNK_SIZE, Chunk};

use crate::debug_tools::{AppDebugOptionExt, DebugGizmos, option_enabled};

const GIZMO_COLOR_CHUNK: f32 = 0.4;
const GIZMO_COLOR_TILE: f32 = 0.2;

const DEBUG_OPTION_GRID: &str = "grid_tiles";

pub(super) struct DebugGridPlugin;

impl Plugin for DebugGridPlugin {
    fn build(&self, app: &mut App) {
        app.register_debug_option(DEBUG_OPTION_GRID).add_systems(
            Update,
            draw_grid_tiles.run_if(option_enabled(DEBUG_OPTION_GRID)),
        );
    }
}

fn draw_grid_tiles(mut gizmos: Gizmos<DebugGizmos>, chunks: Query<&Chunk>) {
    let rotation = Quat::from_rotation_x(90.0f32.to_radians());

    for chunk in chunks.iter() {
        let chunk_position = chunk.position();

        let grid_corner = chunk_position.as_vec2() * CHUNK_SIZE as f32;
        let grid_center = grid_corner + Vec2::splat(CHUNK_SIZE as f32 / 2.0);
        let grid_center = vec3(grid_center.x, -0.02, grid_center.y);

        gizmos
            .grid(
                Isometry3d::new(grid_center, rotation),
                UVec2::splat(CHUNK_SIZE as u32),
                Vec2::splat(1.0),
                Srgba::gray(GIZMO_COLOR_TILE),
            )
            .outer_edges();

        // overlap tile grid
        let grid_center = grid_center + vec3(0., -0.01, 0.);
        gizmos.rect(
            Isometry3d::new(grid_center, rotation),
            Vec2::splat(CHUNK_SIZE as f32),
            Srgba::gray(GIZMO_COLOR_CHUNK),
        );
    }
}
