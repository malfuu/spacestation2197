mod particles;

use bevy::prelude::*;

use crate::game::atmos::particles::ClientAtmosParticlesPlugin;
use atmos::engine::AtmosphericsResource;

pub(super) struct ClientAtmosPlugin;

impl Plugin for ClientAtmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, disable_atmos_simulation)
            .add_plugins(ClientAtmosParticlesPlugin);
    }
}

fn disable_atmos_simulation(mut atmos_resource: ResMut<AtmosphericsResource>) {
    atmos_resource.enabled = false;
}
