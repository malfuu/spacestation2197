use bevy::prelude::*;

use atmos_simulation::prelude::*;

pub(super) struct ClientAtmosPlugin;

impl Plugin for ClientAtmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, disable_atmos_simulation);
    }
}

fn disable_atmos_simulation(mut atmos_resource: ResMut<AtmosphericsResource>) {
    atmos_resource.enabled = false;
}
