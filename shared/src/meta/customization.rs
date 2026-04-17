use rand::{Rng, rng, seq::IndexedRandom};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_replicon::prelude::*;

use crate::placeholder::NAMES;

pub(super) struct CustomizationPlugin;

impl Plugin for CustomizationPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<PlayerSettings>()
            .add_client_message::<SetCustomizationInput>(Channel::Unordered);
    }
}

// yep we are encoding it all in strings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSettings {
    pub name: String,
    pub skin_color: Srgba,
    pub antagonist_candidate: bool,
}

impl CharacterSettings {
    pub fn random() -> Self {
        let mut rng = rng();

        let name = NAMES
            .choose(&mut rng)
            .expect("names isnt empty")
            .to_string();

        let skin_color = Srgba::new(rng.random(), rng.random(), rng.random(), 1.0);

        Self {
            name,
            skin_color,
            antagonist_candidate: true, // should be off when project is more mature.
        }
    }
}

/// Player saved Character Settings
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub struct PlayerSettings {
    pub character: CharacterSettings,
}

impl PlayerSettings {
    pub fn random() -> Self {
        PlayerSettings {
            character: CharacterSettings::random(),
        }
    }
}

/// Client to Server Message of new settings
#[derive(Message, Debug, Serialize, Deserialize, Clone)]
pub struct SetCustomizationInput(pub CharacterSettings);
