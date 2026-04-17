use bevy::prelude::*;
use shared::game::interact::intent::Intent;

use crate::game::mind::MindState;

use super::PlayerIntent;

pub const INTENT_PASSIVE_COLOR: Color = Color::srgb(0.8, 0.8, 1.0);
pub const INTENT_AGGRESSIVE_COLOR: Color = Color::srgb(1.0, 0.4, 0.4);

pub(super) struct ClientIntentUiPlugin;

impl Plugin for ClientIntentUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MindState::Controlling), create_intent_ui)
            .add_systems(
                Update,
                update_intent_ui.run_if(in_state(MindState::Controlling)),
            )
            .add_systems(OnExit(MindState::Controlling), delete_intent_ui);
    }
}

#[derive(Component)]
struct IntentRootNode;

fn intent_ui_data(intent: &Intent) -> (&str, Color) {
    if intent.is_aggressive() {
        ("Aggressive", INTENT_AGGRESSIVE_COLOR)
    } else {
        ("Passive", INTENT_PASSIVE_COLOR)
    }
}

fn create_intent_ui(mut commands: Commands, intent: Res<PlayerIntent>) {
    let (label, color) = intent_ui_data(&intent.0);

    commands.spawn((
        IntentRootNode,
        Node {
            position_type: PositionType::Absolute,
            right: vw(1.0),
            bottom: vh(1.0),
            ..default()
        },
        Text(label.to_string()),
        TextColor(color),
        BackgroundColor(Srgba::gray(0.1).into()),
    ));
}

fn update_intent_ui(
    intent: Res<PlayerIntent>,
    mut intent_node: Single<(&mut Text, &mut TextColor), With<IntentRootNode>>,
) {
    if !intent.is_changed() {
        return;
    }

    let (label, color) = intent_ui_data(&intent.0);

    let (mut text, mut text_color) = intent_node.into_inner();
    text.0 = label.to_string();
    text_color.0 = color;
}

fn delete_intent_ui(mut commands: Commands, intent_node: Single<Entity, With<IntentRootNode>>) {
    commands.entity(*intent_node).despawn();
}
