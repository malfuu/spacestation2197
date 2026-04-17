use bevy::{
    app::ScheduleRunnerPlugin,
    prelude::*,
    render::{
        RenderPlugin,
        settings::{RenderCreation, WgpuSettings},
    },
};

use server::{ServerPlugin, start_game_instance};
use shared::SharedPlugin;

fn main() {
    let mut app = App::new();

    // TODO: until we can turn off bevy_window in server (cant because of bevy_gltf)
    // - blocked  by 0.19
    // app.add_plugins(
    //     DefaultPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
    //         1.0 / 240.0, // arbitrary
    //     ))),
    // )

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None, // disables the window
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
            .set(RenderPlugin {
                synchronous_pipeline_compilation: true,
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: None,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(ScheduleRunnerPlugin {
        run_mode: bevy::app::RunMode::Loop { wait: None },
    })
    .add_plugins(SharedPlugin)
    .add_plugins(ServerPlugin)
    .add_systems(PostStartup, loadup);

    app.run();
}

fn loadup(mut commands: Commands) {
    commands.run_system_cached(start_game_instance);
}
