use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};

use content::prelude::*;
use server::{IsAuthority, ServerPlugin, start_game_instance};
use shared::SharedPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 120.0, // arbitrary
        ))),
    )
    .add_plugins(SharedPlugin)
    .add_plugins(ServerPlugin)
    .add_systems(PostStartup, loadup);

    app.run();
}

fn loadup(mut commands: Commands) {
    commands.insert_resource(IsAuthority::new(true));
    commands.run_system_cached(start_game_instance);
    commands.run_system_cached(log_content_hash);
}

fn log_content_hash(content_hash: Res<ContentHash>) {
    info!("Context Hash: {}", content_hash.into_inner());
}
