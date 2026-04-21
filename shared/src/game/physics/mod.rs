use avian3d::prelude::*;
use bevy::prelude::*;

use bevy_replicon::prelude::*;
use content::prelude::*;

const MAX_LINEAR_SPEED: f32 = 100.0; // arbitrary
const MAX_ANGULAR_SPEED: f32 = 314.15; // arbitrary

pub(super) struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<RigidBody>()
            .prototype_component::<ColliderConstructor>()
            .prototype_component::<Mass>()
            .prototype_component::<CollisionLayers>()
            .replicate::<RigidBody>()
            .replicate::<Collider>()
            .replicate::<CollisionLayers>()
            .add_observer(add_max_velocities)
        ;
    }
}

// reason we dont use avian3d's PhysicLayer is to
// take advantage of const func from_bits
// to keep it simple -
// LAYER prefix is just bits
// LAYER suffix is a full CollisionLayers
// ... yeah sorry for the confusion.

pub const LAYER_NONE: u32 = 0;
// world stuff
pub const LAYER_NORMAL: u32 = 1 << 0;
const _: u32 = 1 << 1;
const _: u32 = 1 << 2;
const _: u32 = 1 << 3;
const _: u32 = 1 << 4;
const _: u32 = 1 << 5;
pub const LAYER_GHOST: u32 = 1 << 6;
pub const LAYER_GHOST_BLOCKING: u32 = 1 << 7;
pub const LAYER_ALL: u32 = u32::MAX;

pub const NORMAL_LAYER: CollisionLayers = CollisionLayers::from_bits(LAYER_NORMAL, LAYER_NORMAL);
pub const GHOST_LAYER: CollisionLayers =
    CollisionLayers::from_bits(LAYER_GHOST, LAYER_GHOST_BLOCKING);

/// Game boundary layers (e.g. floor & ceiling)
pub const BOUNDARY_LAYER: CollisionLayers = CollisionLayers::ALL;

fn add_max_velocities(
    add: On<Add, RigidBody>,
    mut commands: Commands,
) {
    commands.entity(add.entity).insert((
        MaxLinearSpeed(MAX_LINEAR_SPEED),
        MaxAngularSpeed(MAX_ANGULAR_SPEED)
    ));
}
