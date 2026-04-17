//! Mob movement controller, using move-and-slide
// NOTE: my biggest gripe right now is how it stops when it hits dynamic entities
use avian3d::{math::AdjustPrecision, prelude::*};
use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    game::mob::health::Dead,
    utils::{face_direction, filters::MobFilter},
};

const MOVE_AND_SLIDE_ITERATIONS: usize = 2;

#[derive(Component, Default)]
pub struct AccumulatedInput {
    pub last: Option<Vec2>,
}

pub(super) fn clear_accumulated_input(mut accumulated_inputs: Query<&mut AccumulatedInput>) {
    for mut accumulated_input in accumulated_inputs.iter_mut() {
        accumulated_input.last = default();
    }
}

#[derive(Component, Deserialize, Reflect, Clone)]
#[require(Transform, AccumulatedInput, CollidingEntities::default())]
#[reflect(Component)]
pub struct MobController {
    pub speed: f32,
    pub friction: f32,
}

impl Default for MobController {
    fn default() -> Self {
        Self {
            speed: 6.0,
            friction: 20.0,
        }
    }
}

pub(super) fn controller_update_velocity(
    time: Res<Time<Fixed>>,
    mut mobs: Query<
        (&mut LinearVelocity, &AccumulatedInput, &MobController),
        (MobFilter, Without<Dead>),
    >,
) {
    for (mut linear_velocity, input, controller) in mobs.iter_mut() {
        let direction = input.last.unwrap_or(Vec2::ZERO);

        let move_dir = direction.normalize_or_zero();

        let pre_movement_velocity = move_dir * controller.speed;
        let movement_velocity = Vec3::new(pre_movement_velocity.x, 0.0, pre_movement_velocity.y);

        linear_velocity.0 += movement_velocity.adjust_precision();

        let current_speed = linear_velocity.length();
        if current_speed > 0.0 {
            linear_velocity.0 = linear_velocity.0 / current_speed
                * (current_speed
                    - current_speed * controller.friction * time.delta_secs().adjust_precision())
                .max(0.0)
        }
    }
}

pub(super) fn controller_update_rotation(
    mut mobs: Query<(&mut Rotation, &AccumulatedInput, &MobController), (MobFilter, Without<Dead>)>,
) {
    for (mut rotation, input, _) in mobs.iter_mut() {
        let Some(direction) = input.last else {
            continue;
        };

        let move_dir = direction.normalize_or_zero();

        if move_dir.length_squared() > 0.01 {
            let target = face_direction(move_dir);
            rotation.0 = rotation.0.slerp(target, 0.30);
        }
    }
}

type ControlerMoveAndSlideQueryData<'w> = (
    Entity,
    &'w mut Transform,
    &'w mut LinearVelocity,
    &'w CollisionLayers,
    &'w Collider,
);

pub(super) fn controller_move_and_slide(
    mut query: Query<ControlerMoveAndSlideQueryData, (With<MobController>, Without<Dead>)>,
    move_and_slide: MoveAndSlide,
    time: Res<Time<Fixed>>,
) {
    // NOTE: Taken straight from the moveandslide example!
    for (entity, transform, mut lin_vel, layer, collider) in &mut query {
        let filter = SpatialQueryFilter::from_mask(layer.filters).with_excluded_entities([entity]);

        let MoveAndSlideOutput {
            position: _position,
            projected_velocity,
        } = move_and_slide.move_and_slide(
            collider,
            transform.translation.adjust_precision(),
            transform.rotation.adjust_precision(),
            lin_vel.0,
            time.delta(),
            &MoveAndSlideConfig {
                move_and_slide_iterations: MOVE_AND_SLIDE_ITERATIONS,
                ..default()
            },
            &filter,
            |_| MoveAndSlideHitResponse::Accept,
        );

        // Update transform and velocity
        // transform.translation = position.f32();
        lin_vel.0 = projected_velocity;
    }
}
