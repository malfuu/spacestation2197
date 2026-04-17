use bevy::prelude::*;

use shared::game::light::Light;

pub(super) struct ClientLightPlugin;

impl Plugin for ClientLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_sun)
            .add_observer(on_light_add);
    }
}

fn create_sun(mut commands: Commands) {
    commands.spawn((
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                -80.0_f32.to_radians(),
                20.0_f32.to_radians(),
                0.0,
            ),
            ..default()
        },
        DirectionalLight {
            illuminance: light_consts::lux::OFFICE,
            shadows_enabled: true,
            ..default()
        },
    ));

    // commands.insert_resource(GlobalAmbientLight::NONE);
}

fn on_light_add(on: On<Add, Light>, mut commands: Commands, lights: Query<&Light>) {
    let entity = on.entity;
    let light = lights.get(entity).expect("light should exist");

    commands.entity(entity).insert(PointLight {
        color: light.color.into(),
        intensity: light.intensity,
        range: light.range,
        radius: 2.0,
        shadows_enabled: true,
        ..default()
    });
}
