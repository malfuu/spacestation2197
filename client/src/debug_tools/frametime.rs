use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig}, diagnostic::DiagnosticsStore, prelude::*
};

pub(super) struct DebugFrametimePlugin;

impl Plugin for DebugFrametimePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                enabled: false,
                frame_time_graph_config: FrameTimeGraphConfig {
                    enabled: false,
                    min_fps: 60.0,
                    target_fps: 480.0,
                },
                ..default()
            },
        });
    }
}

fn test(
    res: Res<DiagnosticsStore>
) {

}
