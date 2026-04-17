use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_replicon::prelude::*;

use crate::{meta::manager::Manager, utils::filters::ManagerFilter};

pub(super) struct RoundPlugin;

impl Plugin for RoundPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<RoundState>()
            .replicate::<StartRoundTimer>()
            .replicate::<RoundEndTimer>()
            .replicate::<Ready>()
            .add_client_message::<JoinInput>(Channel::Unordered)
            .add_client_message::<ReadyInput>(Channel::Unordered);
    }
}

/// Mark a player entity as ready
#[derive(Component, Serialize, Deserialize)]
pub struct Ready;

/// Sets lobby count down
#[derive(Event, Serialize, Deserialize)]
pub struct SetStartTimer;

/// Triggered when the lobby timer has finished,
/// does not necessarily mean round will start.
#[derive(Event, Serialize, Deserialize)]
pub struct StartTimerFinished;

/// Triggered when the round has started.
#[derive(Event, Serialize, Deserialize)]
pub struct RoundStartedEvent;

/// Triggers round end; no going back
#[derive(Event, Serialize, Deserialize)]
pub struct EndRoundEvent;

#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
#[require(Manager)]
pub struct StartRoundTimer(pub Timer);

#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
#[require(Manager)]
pub struct RoundEndTimer(pub Timer);

#[derive(Component, Debug, Serialize, Deserialize)]
#[require(Manager)]
#[component(immutable)]
pub enum RoundState {
    Starting,
    Ongoing,
    Ended,
}

/// Lobby Inputs, false if observing, true if joining fr
#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct JoinInput(pub bool);

/// True if readying, false if unreadying.
#[derive(Message, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ReadyInput(pub bool);

/// run condition for if round is starting
pub fn is_round_starting(state: Single<&RoundState, ManagerFilter>) -> bool {
    matches!(*state, RoundState::Starting)
}

/// run condition for if round is ingoing
pub fn is_round_ongoing(state: Single<&RoundState, ManagerFilter>) -> bool {
    matches!(*state, RoundState::Ongoing)
}

/// run condition for if round has ended
pub fn is_round_finished(state: Single<&RoundState, ManagerFilter>) -> bool {
    matches!(*state, RoundState::Ended)
}
