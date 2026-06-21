//! Stuff that needs to be transfered to scripting domain later
//! or just placeholder misc stuff

use avian3d::prelude::*;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use serde::Deserialize;

use content::prelude::*;

pub(super) struct PlaceholderPlugin;

impl Plugin for PlaceholderPlugin {
    fn build(&self, app: &mut App) {
        app.prototype_component::<MobCollider>()
            .prototype_component::<ItemCollider>();
    }
}

/// Random names for characters until we read setup databases for random names.
pub const NAMES: [&str; 24] = [
    "Avery", "Rowan", "Quinn", "Jordan", "Taylor", "Morgan", "Skyler", "Reese", "River", "Sage",
    "Casey", "Jamie", "Alex", "Drew", "Robin", "Kendall", "Blair", "Indigo", "Ocean", "Briar",
    "Wren", "Storm", "Rio", "Lee",
];

// HACK: below are a serious of components to get around scripting's lack of support for component
// customization. Until better solutions arrives this will stay here.

fn on_mob_collider_add(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    world
        .commands()
        .entity(entity)
        .insert((LockedAxes::ROTATION_LOCKED.lock_translation_y(),));
}

#[derive(Component, Deserialize, Reflect, Default, Clone)]
#[component(
    on_add = on_mob_collider_add
)]
#[require(
    Collider::compound(vec![(
        Vec3::new(0.0, 0.85, 0.0),
        Quat::IDENTITY,
        Collider::cylinder(0.25, 1.7),
    )]),
)]
#[reflect(Component)]
pub struct MobCollider;

#[derive(Component, Deserialize, Reflect, Default, Clone)]
#[require(
    Collider::cuboid(0.25, 0.5, 0.25),
    // CollisionLayers::from_bits(LAYER_NORMAL, LAYER_NORMAL), TODO return bits
)]
#[reflect(Component, Default)]
pub struct ItemCollider;
