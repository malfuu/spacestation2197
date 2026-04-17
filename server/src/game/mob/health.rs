use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use bevy::prelude::*;
use shared::{
    game::mob::health::{Dead, Death, Health},
    utils::{filters::MobFilter, physics::PhysicsEntityCommandsExt},
};

pub(super) fn die(
    mut commands: Commands,
    mobs: Query<(Entity, &Health), (MobFilter, Without<Dead>)>,
) {
    for (entity, health) in mobs.iter() {
        if !health.is_lethal() {
            continue;
        }

        info!("Mob {entity:?} has died!");

        commands.trigger(Death { target: entity });
    }
}

pub(super) fn on_death(
    death: On<Death>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Rotation, &mut LinearVelocity), MobFilter>,
) {
    let Ok((mut transform, mut rotation, mut velocity)) = query.get_mut(death.target) else {
        return;
    };

    velocity.0 = Vec3::ZERO;

    commands
        .entity(death.target)
        .insert(Dead)
        .insert(
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
        )
        .disable_physics();

    let mut target_rotation = Quat::IDENTITY;
    target_rotation *= Quat::from_rotation_z(FRAC_PI_2);
    target_rotation *= Quat::from_rotation_y(FRAC_PI_2);

    transform.rotation = target_rotation;
    rotation.0 = target_rotation;
}
