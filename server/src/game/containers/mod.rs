use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{game::containers::Contained, utils::physics::PhysicsEntityCommandsExt};

pub(super) struct ContainersPlugin;

impl Plugin for ContainersPlugin {
    fn build(&self, app: &mut App) {
        app.sync_related_entities::<Contained>()
            .add_observer(on_contained_add)
            .add_observer(on_contained_removed);
    }
}

fn on_contained_add(
    add: On<Add, Contained>,
    mut commands: Commands,
    mut position_rotation: Query<(&mut Transform, &mut Position, &mut Rotation), With<Contained>>,
) {
    if let Ok((mut transform, mut position, mut rotation)) = position_rotation.get_mut(add.entity) {
        *transform = Transform::IDENTITY;
        *position = Position::new(Vec3::ZERO);
        *rotation = Rotation::IDENTITY;
    }

    commands.entity(add.entity).disable_physics();
}

fn on_contained_removed(remove: On<Remove, Contained>, mut commands: Commands) {
    commands.entity(remove.entity).enable_physics();
}
