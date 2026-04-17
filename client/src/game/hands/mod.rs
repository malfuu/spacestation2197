use bevy::prelude::*;
use common::EntityTag;
use shared::game::hands::{DropInput, Hand, Hands, SwitchHandsInput, UseInput};

use crate::game::mind::{Controlling, MindState};

pub const HAND_ACTIVE_COLOR: Color = Color::srgb(1., 1., 0.);

pub(super) struct ClientHandsPlugin;

impl Plugin for ClientHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (hands_input, update_hands).run_if(in_state(MindState::Controlling)),
        )
        .add_systems(OnEnter(MindState::Controlling), create_hands)
        .add_systems(OnExit(MindState::Controlling), delete_hands);
    }
}

/// marker identifier for hands root node
#[derive(Component)]
struct HandsRootNode;

/// entity marker identifier
#[derive(Component)]
struct HandText(Hand);

fn hands_input(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyQ) {
        commands.write_message(DropInput);
    }

    if keys.just_pressed(KeyCode::KeyZ) {
        commands.write_message(UseInput);
    }

    if keys.just_pressed(KeyCode::KeyX) {
        commands.write_message(SwitchHandsInput);
    }
}

fn create_hands(mut commands: Commands, hands: Single<Option<&Hands>, With<Controlling>>) {
    if hands.is_none() {
        return;
    };

    commands.spawn((
        HandsRootNode,
        Node {
            position_type: PositionType::Absolute,
            left: vw(0.0),
            right: vw(1.0),
            bottom: vh(1.0),
            width: percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        children![
            (
                HandText(Hand::Left),
                Text("left".to_string()),
                TextColor::default(),
                BackgroundColor(Srgba::gray(0.2).into()),
            ),
            (
                HandText(Hand::Right),
                Text("right".to_string()),
                TextColor::default(),
                BackgroundColor(Srgba::gray(0.2).into()),
            ),
        ],
    ));
}

fn hand_text(held: Option<Entity>, items: &Query<&EntityTag>) -> String {
    held.map(|v| {
        let tag = items.get(v).expect("Entity should have a tag.");
        (**tag).to_string()
    })
    .unwrap_or_else(|| "Empty".to_string())
}

fn update_hands(
    hands: Single<Ref<Hands>, With<Controlling>>,
    items: Query<&EntityTag>,
    mut ui_query: Query<(&HandText, &mut Text, &mut TextColor)>,
) {
    if !hands.is_changed() {
        return;
    }

    let left_label = hand_text(hands.get(Hand::Left), &items);
    let right_label = hand_text(hands.get(Hand::Right), &items);

    for (hand_marker, mut text, mut color) in ui_query.iter_mut() {
        match hand_marker.0 {
            Hand::Left => {
                text.0 = left_label.clone();
                color.0 = if matches!(hands.active, Hand::Left) {
                    HAND_ACTIVE_COLOR
                } else {
                    TextColor::default().0
                };
            }
            Hand::Right => {
                text.0 = right_label.clone();
                color.0 = if matches!(hands.active, Hand::Right) {
                    HAND_ACTIVE_COLOR
                } else {
                    TextColor::default().0
                };
            }
        }
    }
}

fn delete_hands(mut commands: Commands, query: Query<Entity, With<HandsRootNode>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
