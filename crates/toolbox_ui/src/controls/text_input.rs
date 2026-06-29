use bevy::prelude::*;
use bevy::text::{EditableText, LineBreak, TextCursorStyle, TextLayout};

/// Container for text input field.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct TextInputContainer;

impl TextInputContainer {
    pub fn scene() -> impl Scene {
        bsn! {
            Node {
                height: px(28.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(6.0)),
                border_radius: BorderRadius::all(px(4.0)),
                flex_grow: 1.0,
            }
            TextInputContainer
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15))
            BorderColor::all(Color::srgb(0.30, 0.30, 0.35))
        }
    }
}

/// Properties accepted by TextInput.
#[derive(Default, Clone)]
pub struct TextInputProps {
    pub visible_width: Option<f32>,
    pub max_characters: Option<usize>,
}

/// The TextInput Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(TextInputProps)]
#[reflect(Component, Default)]
pub struct TextInput;

impl TextInput {
    pub fn scene(props: TextInputProps) -> impl Scene {
        bsn! {
            Node {
                flex_grow: 1.0,
            }
            TextInput
            EditableText {
                cursor_width: 0.05,
                visible_width: {props.visible_width},
                max_characters: {props.max_characters},
            }
            TextLayout {
                linebreak: LineBreak::NoWrap,
            }
            TextCursorStyle::default()
            Text::new("")
            TextColor(Color::srgb(0.90, 0.90, 0.95))
        }
    }
}

/// Plugin registering systems for text inputs.
pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, _app: &mut App) {}
}
