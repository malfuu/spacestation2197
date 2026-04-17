use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    audio::AudioCommandsExt,
    meta::{
        MetaSystems,
        gamemode::Gamemode,
        round::{
            EndRoundEvent, JoinInput, Ready, ReadyInput, RoundEndTimer, RoundStartedEvent,
            RoundState, SetStartTimer, StartRoundTimer, StartTimerFinished, is_round_finished,
            is_round_starting,
        },
    },
    utils::{
        ServerSettings,
        filters::{ManagerFilter, PlayerFilter},
    },
};

use crate::utils::{SpawnMethod, SpawnerCommandsExt};

pub(crate) struct RoundPlugin;

impl Plugin for RoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_start_round_timer)
            .add_observer(on_start_timer_finished)
            .add_observer(on_round_start)
            .add_observer(on_round_end)
            .add_systems(
                FixedUpdate,
                (read_ready, read_join).in_set(MetaSystems::Inputs),
            )
            .add_systems(
                FixedUpdate,
                check_lobby_timer
                    .in_set(MetaSystems::Logic)
                    .run_if(is_round_starting),
            )
            .add_systems(
                FixedUpdate,
                check_round_end_timer
                    .in_set(MetaSystems::Logic)
                    .run_if(is_round_finished),
            );
    }
}

pub(super) fn read_ready(
    mut reader: MessageReader<FromClient<ReadyInput>>,
    mut commands: Commands,
) {
    for input in reader.read() {
        let ClientId::Client(client_entity) = input.client_id else {
            continue;
        };

        let Ok(mut entity) = commands.get_entity(client_entity) else {
            warn!("No existing player entity for {client_entity:?}");
            continue;
        };

        if input.0 {
            entity.insert(Ready);
        } else {
            entity.remove::<Ready>();
        }
    }
}

pub(super) fn read_join(mut reader: MessageReader<FromClient<JoinInput>>, mut commands: Commands) {
    for input in reader.read() {
        let ClientId::Client(client_entity) = input.client_id else {
            continue;
        };

        let Ok(_) = commands.get_entity(client_entity) else {
            warn!("No existing player entity for {client_entity:?}");
            continue;
        };

        // TODO: check if client already has mob

        let is_observing = input.0;
        let (proto, spawner) = if is_observing {
            ("human", "spawner_human")
        } else {
            ("ghost", "spawner_observer")
        };

        commands.spawn_player(
            client_entity,
            proto.to_string(),
            SpawnMethod::Spawner(spawner.to_string()),
        );
    }
}

fn check_lobby_timer(
    mut commands: Commands,
    state: Single<(Entity, &mut StartRoundTimer), ManagerFilter>,
    time: Res<Time<Fixed>>,
) {
    let (manager_entity, mut timer) = state.into_inner();

    timer.tick(time.delta());

    commands.entity(manager_entity).insert(RoundState::Starting);

    if timer.is_finished() {
        info!("Lobby countdown finished.");
        commands.trigger(StartTimerFinished);
    }
}

fn check_round_end_timer(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut timer: Single<&mut RoundEndTimer, ManagerFilter>,
) {
    timer.tick(time.delta());

    if timer.is_finished() {
        commands.write_message(AppExit::Success);
    }
}

fn on_start_round_timer(
    _: On<SetStartTimer>,
    mut commands: Commands,
    settings: Res<ServerSettings>,
    manager: Single<(Entity, &RoundState), ManagerFilter>,
) {
    let (entity, state) = manager.into_inner();
    if !matches!(state, RoundState::Starting) {
        warn!("SetStartTimer on non starting round state!");
        return;
    }

    commands
        .entity(entity)
        .insert(StartRoundTimer(Timer::from_seconds(
            settings.lobby_timer as f32,
            TimerMode::Once,
        )));
}

fn on_start_timer_finished(
    _: On<StartTimerFinished>,
    mut commands: Commands,
    manager: Single<(Entity, &RoundState), ManagerFilter>,
    players: Query<Entity, (PlayerFilter, With<Ready>)>,
    gamemode: Single<&Gamemode>,
) {
    if matches!(**gamemode, Gamemode::Mafia) && players.count() < 2 {
        info!("Could not start, not enough players!");
        // commands.trigger(SetStartTimer(Duration::from_secs(10)));
        commands.trigger(SetStartTimer);
        return;
    }

    let (manager, state) = manager.into_inner();
    if !matches!(*state, RoundState::Starting) {
        warn!("RoundStart attempted without starting state!");
        return;
    }

    commands.entity(manager).insert(RoundState::Ongoing);

    info!("Starting round!");

    for player in players.iter() {
        commands.spawn_player(
            player,
            "human".to_string(),
            SpawnMethod::Position(Vec2::splat(2.0)),
        );
    }

    commands.trigger(RoundStartedEvent);
}

fn on_round_start(_: On<RoundStartedEvent>, mut commands: Commands) {
    commands.play_sound_globally("sounds/welcome.ogg");
}

fn on_round_end(
    _: On<EndRoundEvent>,
    mut commands: Commands,
    settings: Res<ServerSettings>,
    state: Single<(Entity, &RoundState), ManagerFilter>,
) {
    let end_duration = settings.round_end_timer;
    info!("Round ended!");
    info!("Turning off in {end_duration:.0?} seconds!");

    let (manager, _) = state.into_inner();
    commands.entity(manager).insert((
        RoundState::Ended,
        RoundEndTimer(Timer::from_seconds(end_duration as f32, TimerMode::Once)),
    ));
}
