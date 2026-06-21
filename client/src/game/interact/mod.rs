mod ui;

use bevy::prelude::*;

use shared::{
    audio::PlaySoundMessage,
    game::interact::{InteractInput, intent::Intent},
};

use crate::{base::input::EntityClick, game::interact::ui::ClientIntentUiPlugin};

const SOUND_AGRESSIVE: &str = "sounds/ui_togglecombat.ogg";
const SOUND_PASSIVE: &str = "sounds/ui_toggleoffcombat.ogg";

pub(super) struct ClientInteractPlugin;

impl Plugin for ClientInteractPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientIntentUiPlugin)
            .init_resource::<PlayerIntent>()
            .add_systems(Update, switch_intent)
            .add_observer(on_entity_click);
    }
}

#[derive(Resource, Default)]
pub struct PlayerIntent(pub Intent);

fn switch_intent(
    mut player_intent: ResMut<PlayerIntent>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
) {
    // in the future intent should be server side as well.
    if input.just_pressed(KeyCode::Digit4) {
        let new_intent = player_intent.0.switch();

        let sound_path = match new_intent {
            Intent::Aggressive => SOUND_AGRESSIVE,
            Intent::Passive => SOUND_PASSIVE,
        };

        commands.write_message(PlaySoundMessage {
            sound: sound_path.to_string(),
        });
    }
}

fn on_entity_click(
    click: On<EntityClick>,
    mut commands: Commands,
    player_intent: Res<PlayerIntent>,
) {
    commands.write_message(InteractInput {
        target: click.entity,
        intent: player_intent.0,
    });
}
