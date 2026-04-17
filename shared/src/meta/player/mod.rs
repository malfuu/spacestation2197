use bevy::prelude::*;
use rand::{rng, seq::IndexedRandom};
use serde::{Deserialize, Serialize};

use bevy_replicon::{prelude::*, shared::backend::connected_client::NetworkId};

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.replicate_once::<NetworkId>()
            .replicate_once::<PlayerName>()
            .replicate::<Ping>();
    }
}

/// Marker for player entities
#[derive(Component, Default, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub struct Player;

const NATO: &[&str] = &[
    "Alfa", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel", "India", "Juliett",
    "Kilo", "Lima", "Mike", "November", "Oscar", "Papa", "Quebec", "Romeo", "Sierra", "Tango",
    "Uniform", "Victor", "Whiskey", "X-ray", "Yankee", "Zulu",
];

#[derive(Component, Default, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub struct PlayerName {
    name: String,
}

impl PlayerName {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get(&self) -> &String {
        &self.name
    }

    pub fn random() -> Self {
        let name = *NATO.choose(&mut rng()).unwrap();
        Self { name: name.into() }
    }
}

/// Ping of a player entity
#[derive(Component, Serialize, Deserialize, Default, Clone)]
#[component(immutable)]
pub struct Ping {
    pub rtt: f32,
}
