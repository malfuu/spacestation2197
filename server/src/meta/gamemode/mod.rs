use bevy::prelude::*;

use shared::{
    game::{player::Player, sandbox::Sandboxer},
    meta::{
        MetaSystems,
        gamemode::Gamemode,
        round::{RoundStartedEvent, is_round_ongoing},
    },
};

/// how many crewmembers per traitor
pub const TRAITOR_RATIO: usize = 4;

pub(super) struct ServerGamemodePlugin;

impl Plugin for ServerGamemodePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_round_start);
    }
}

#[derive(Event, Debug)]
pub enum Victory {
    Traitors,
    Crewmembers,
    Tasks,
}

/// marks a [`shared::game::mob::Mob`] as traitor
#[derive(Component)]
pub struct Traitor;

fn on_round_start(
    _: On<RoundStartedEvent>,
    gamemode: Single<&Gamemode>,
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
) {
    match **gamemode {
        Gamemode::Extended => {}
        Gamemode::Sandbox => {
            for entity in players.iter() {
                commands.entity(entity).insert(Sandboxer);
            }
        }
    }
}
