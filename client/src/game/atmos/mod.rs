mod particles;

use bevy::prelude::*;

use atmos_simulation::prelude::*;

#[cfg(not(feature = "no-atmos-particles"))]
use crate::game::atmos::particles::AtmosParticlesPlugin;

pub(super) struct ClientAtmosPlugin;

impl Plugin for ClientAtmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, disable_atmos_simulation);

        #[cfg(not(feature = "no-atmos-particles"))]
        app.add_plugins(AtmosParticlesPlugin);
    }
}

fn disable_atmos_simulation(mut atmos_resource: ResMut<AtmosphericsResource>) {
    atmos_resource.enabled = false;
}
