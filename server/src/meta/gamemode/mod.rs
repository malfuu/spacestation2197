use bevy::prelude::*;

use rand::seq::IteratorRandom;
use shared::{
    InstanceState,
    audio::AudioCommandsExt,
    game::{player::Player, sandbox::Sandboxer},
    meta::{
        MetaSystems,
        gamemode::Gamemode,
        round::{EndRoundEvent, RoundStartedEvent, is_round_ongoing},
    },
    utils::filters::MobFilter,
};

use crate::{game::mind::Controlled, showcase::TaskManager, utils::MessageCommandsExt};

/// how many crewmembers per traitor
pub const TRAITOR_RATIO: usize = 4;

pub(super) struct ServerGamemodePlugin;

impl Plugin for ServerGamemodePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (check_round_end)
                .in_set(MetaSystems::Logic)
                .run_if(is_round_ongoing),
        )
        .add_observer(on_round_start)
        .add_observer(on_victory);
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
    _state: Res<State<InstanceState>>,
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    mobs: Query<(Entity, &Controlled)>,
) {
    match **gamemode {
        Gamemode::Extended => {}
        Gamemode::Sandbox => {
            for entity in players.iter() {
                commands.entity(entity).insert(Sandboxer);
            }
        }
        Gamemode::Mafia => {
            info!("Mafia mode started");
            let mob_count = mobs.iter().count();
            let traitor_count = (mob_count / TRAITOR_RATIO).max(1);

            let mut mob_pool = Vec::new();
            for (entity, _) in mobs.iter() {
                mob_pool.push(entity);
            }

            let chosen_traitors: std::collections::HashSet<_> = mob_pool
                .into_iter()
                .choose_multiple(&mut rand::rng(), traitor_count)
                .into_iter()
                .collect();

            for (mob_entity, controlled) in mobs.iter() {
                let player_entity = **controlled;

                if chosen_traitors.contains(&mob_entity) {
                    commands.entity(mob_entity).insert(Traitor);
                    commands.send_chat_message(
                        player_entity,
                        "You are the traitor! Kill everyone else and prevent tasks from being done."
                    );
                } else {
                    commands.send_chat_message(
                        player_entity,
                        "You are a crewmember, perform your tasks!",
                    );
                }
            }
        }
    }
}

fn check_round_end(
    gamemode: Single<&Gamemode>,
    mut commands: Commands,
    manager: Res<TaskManager>,
    traitors: Query<Entity, (With<Traitor>, MobFilter)>,
    crewmembers: Query<Entity, (Without<Traitor>, MobFilter)>,
) {
    match **gamemode {
        Gamemode::Extended | Gamemode::Sandbox => {}
        Gamemode::Mafia => {
            let traitor_count = traitors.iter().count();
            let crew_count = crewmembers.iter().count();

            let victory = if crew_count == 0 {
                Victory::Traitors
            } else if traitor_count == 0 {
                Victory::Crewmembers
            } else if manager.all_tasks_done() {
                Victory::Tasks
            } else {
                return;
            };

            commands.trigger(victory);
        }
    }
}

fn on_victory(victory: On<Victory>, mut commands: Commands) {
    let message = match *victory {
        Victory::Traitors => "Traitors win! All crewmembers are dead.".to_string(),
        Victory::Crewmembers => "Crewmembers win, with no traitors left alive!".to_string(),
        Victory::Tasks => {
            "Crewmembers win! Traitors were unable to stop all tasks from being done!".to_string()
        }
    };
    let message = format!("Game has finished. {message}");

    info!(message);
    commands.broadcast_chat_message(message);
    commands.play_sound_globally("sounds/bloop.ogg");

    commands.trigger(EndRoundEvent);
}
