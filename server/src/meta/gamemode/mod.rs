use bevy::prelude::*;

use shared::{
    game::{player::Player, sandbox::Sandboxer},
    meta::{gamemode::Gamemode, round::RoundStartedEvent},
};

pub(super) struct ServerGamemodePlugin;

impl Plugin for ServerGamemodePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_round_start);
    }
}

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
