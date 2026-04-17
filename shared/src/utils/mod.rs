//! dump of util functions.
//! what is a project without them? an organized one.
pub mod filters;
pub mod physics;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{defines::DEFAULT_LISTEN_PORT, meta::gamemode::Gamemode};

const DEFAULT_SERVER_NAME: &str = "ss2917";
const DEFAULT_MAP_NAME: &str = "ministation";

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    pub server_name: String,
    pub tick_rate: u32,
    pub max_players: usize,

    pub port: u16,
    pub address: String,
    pub password: Option<String>,
    pub make_local_admin: bool,

    pub lobby_timer: u32,
    pub round_end_timer: u32,
    pub lobby_enabled: bool,

    pub map_name: String,

    pub atmos_enabled: bool,

    pub gamemode: Gamemode,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            server_name: DEFAULT_SERVER_NAME.to_string(),
            tick_rate: 30,
            max_players: 64,

            port: DEFAULT_LISTEN_PORT,
            address: "127.0.0.1".to_string(),
            password: None,
            make_local_admin: false,

            lobby_timer: 120,
            round_end_timer: 60,
            lobby_enabled: true,

            map_name: DEFAULT_MAP_NAME.to_string(),

            atmos_enabled: true,

            gamemode: Gamemode::Extended,
        }
    }
}

impl ServerSettings {
    pub fn load_from_file(path: &str) -> Self {
        let Ok(contents) = fs::read_to_string(path) else {
            error!("Could not read {}", path);
            panic!("Failed to read settings file");
        };

        let Ok(settings) = toml::from_str(&contents) else {
            error!("Failed to parse {}", path);
            panic!("Failed to parse settings file");
        };

        info!("Successfully loaded settings from {}", path);
        settings
    }
}

pub fn face_direction(direction: Vec2) -> Quat {
    let normalized = direction.normalize();
    let angle = normalized.x.atan2(normalized.y);
    Quat::from_rotation_y(angle)
}
