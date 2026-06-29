use bevy::prelude::*;
use bevy::ui_widgets::Checkbox as BevyCheckbox;

/// The DisclosureToggle Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct DisclosureToggle;

impl DisclosureToggle {
    pub fn scene() -> impl Scene {
        bsn!(
            Node {
                width: px(12.0),
                height: px(12.0),
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
            BevyCheckbox
            DisclosureToggle
            Children [(
                Node {
                    width: px(8.0),
                    height: px(8.0),
                }
                BackgroundColor(Color::srgb(0.70, 0.70, 0.75))
            )]
        )
    }
}

/// Plugin registering systems for disclosure toggles.
pub struct DisclosureTogglePlugin;

impl Plugin for DisclosureTogglePlugin {
    fn build(&self, _app: &mut App) {}
}
