use bevy::prelude::*;

use bevy_replicon::{prelude::*, shared::backend::connected_client::NetworkId};
use serde::{Deserialize, Serialize};

use crate::{game::sandbox::Sandboxer, meta::gamemode::Gamemode};

pub(super) struct AdministrationPlugin;

impl Plugin for AdministrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_message::<AdminCommandMessage>(Channel::Ordered);
    }
}

/// Marks a player entity as administrator
#[derive(Component, Clone, Copy)]
#[component(immutable)]
#[require(Sandboxer)]
pub struct Administrator;

#[derive(Message, Serialize, Deserialize)]
pub enum AdminCommandMessage {
    // Server controls
    /// Sets the refresh rate of [`FixedUpdate`]
    SetTps(u32),
    /// Turns atmos schedule on or off
    SetAtmos(bool),
    /// Turns physics schedule on or off
    SetPhysics(bool),
    /// Turns gameplay systems on or off, effectively freezing the game.
    SetGameplay(bool),
    /// Turns off or enables OOC.
    SetOoc(bool),

    // player commands
    /// Send a BWOINK to a player
    AdminMessage(NetworkId),
    /// Kicks a player given their NetworkId
    Kick(NetworkId),
    /// Kicks a player given their NetworkId
    Ban(NetworkId),
    /// Boots the player from any mobs (incl. ghosts), effectively sending them to the lobby.
    Respawn(NetworkId),

    // round commands
    /// If the round is starting, it forces a start
    ForceStartRound,
    /// If the round is ongoing, it forces round end.
    ForceEndRound,
    /// Shutdowns the server. Full stop.
    Shutdown,
    /// Sets gamemode, only works before the round has started.
    SetGamemode(Gamemode),
}
