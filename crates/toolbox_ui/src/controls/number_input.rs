use crate::controls::text_input::{TextInput, TextInputContainer};
use bevy::prelude::*;

/// Format of number for input.
#[derive(Component, Default, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub enum NumberFormat {
    #[default]
    F32,
    F64,
    I32,
    I64,
}

/// Properties accepted by NumberInput.
pub struct NumberInputProps {
    pub label_text: Option<&'static str>,
    pub number_format: NumberFormat,
}

impl Default for NumberInputProps {
    fn default() -> Self {
        Self {
            label_text: None,
            number_format: NumberFormat::F32,
        }
    }
}

/// The NumberInput Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(NumberInputProps)]
#[reflect(Component, Default)]
pub struct NumberInput;

impl NumberInput {
    pub fn scene(props: NumberInputProps) -> impl Scene {
        bsn! {
            @TextInputContainer
            NumberInput
            template_value(props.number_format)
            Children [
                {
                    match props.label_text {
                        Some(text) => Box::new(bsn_list!(
                            Node {
                                display: Display::Flex,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(px(4.0)),
                            }
                            Text(text)
                            TextColor(Color::srgb(0.70, 0.70, 0.75))
                        )) as Box<dyn SceneList>,
                        None => Box::new(bsn_list!()) as Box<dyn SceneList>
                    }
                },
                @TextInput
            ]
        }
    }
}
