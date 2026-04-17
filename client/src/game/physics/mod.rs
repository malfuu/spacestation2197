use bevy::prelude::*;

use avian3d::prelude::*;

pub(super) struct ClientPhysicsPlugin;

impl Plugin for ClientPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, disable_physics_simulation);
    }
}

fn disable_physics_simulation(mut time: ResMut<Time<Physics>>) {
    // we are not doing client side simulation... yet.
    time.pause();
}
