use bevy::prelude::*;

use shared::game::containers::Contained;

pub(super) struct ClientContainerPlugin;

impl Plugin for ClientContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_contained_add)
            .add_observer(on_contained_removed);
    }
}

fn on_contained_add(add: On<Add, Contained>, mut commands: Commands) {
    commands.entity(add.entity).insert(Visibility::Hidden);
}

fn on_contained_removed(add: On<Remove, Contained>, mut query: Query<&mut Visibility>) {
    let Ok(mut visibility) = query.get_mut(add.entity) else {
        return;
    };

    *visibility = Visibility::Inherited;
}
