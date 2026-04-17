use bevy::prelude::*;

use crate::{
    base::camera::CameraState,
    game::mind::{Controlling, MindState},
};

pub(super) struct ClientCameraPlugin;

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MindState::Controlling), on_enter_mob)
            .add_systems(OnExit(MindState::Controlling), on_exit_mob);
    }
}

fn on_enter_mob(
    mut commands: Commands,
    mut mode: ResMut<CameraState>,
    mob: Single<Entity, With<Controlling>>,
) {
    *mode = CameraState::follow(*mob);
}

fn on_exit_mob(mut mode: ResMut<CameraState>) {
    *mode = CameraState::default();
}
