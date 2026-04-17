use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use shared::game::{mind::OwnMobMessage, mob::Mob};

use crate::base::session::SessionState;

pub(super) struct ClientMindPlugin;

impl Plugin for ClientMindPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MindState>()
            .init_resource::<ControllingMob>()
            .add_systems(OnExit(SessionState::Playing), on_disconnect)
            .add_systems(Update, read_on_own_mob);
    }
}

/// whether the player is in lobby or controlling a mob
#[derive(SubStates, PartialEq, Eq, Hash, Default, Debug, Clone, Copy)]
#[source(SessionState = SessionState::Playing)]
pub enum MindState {
    #[default]
    NotControlling,
    Controlling,
}

/// Marker for what entity is being controlled by this client. should be single
/// Only one entity with [`Controlling`] at a given time
#[derive(Component)]
#[component(
    on_add = add_spatial_listener,
    on_remove = remove_spatial_listener
)]
pub struct Controlling;

fn add_spatial_listener(mut world: DeferredWorld, ctx: HookContext) {
    world
        .commands()
        .entity(ctx.entity)
        .insert(SpatialListener::new(0.15));
}

/// Hook: Fires when `Controlling` is removed from an entity
fn remove_spatial_listener(mut world: DeferredWorld, ctx: HookContext) {
    world
        .commands()
        .entity(ctx.entity)
        .remove::<SpatialListener>();
}

#[derive(Resource, Deref, Default)]
struct ControllingMob(Option<Entity>);

fn read_on_own_mob(
    mut messages: MessageReader<OwnMobMessage>,
    mut commands: Commands,
    mobs: Query<&Mob>,
    mut resource: ResMut<ControllingMob>,
) {
    for own_mob in messages.read() {
        // FIX: old controlling resource should be cleared.
        if let Some(old_entity) = resource.0.take() {
            if let Ok(mut entity_commands) = commands.get_entity(old_entity) {
                entity_commands.remove::<Controlling>();
            }
            commands.set_state(MindState::NotControlling);
        }

        if let Some(new_mob) = own_mob.0 {
            if !mobs.contains(new_mob) {
                error!("Received OwnMob on a non-mob!");
                return;
            }
            commands.entity(new_mob).insert(Controlling);
            commands.set_state(MindState::Controlling);
        };

        resource.0 = own_mob.0;
    }
}

fn on_disconnect(mut commands: Commands, mut resource: ResMut<ControllingMob>, mobs: Query<&Mob>) {
    resource.0 = None;
    commands.set_state(MindState::NotControlling);
}
