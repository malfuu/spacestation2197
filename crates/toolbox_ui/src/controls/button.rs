use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::Button as BevyButton;
use crate::palette;

/// Color & style variants for the button.
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component, Default)]
pub enum ButtonVariant {
    #[default]
    Normal,
    Primary,
    Plain,
}

/// Properties accepted by the button scene constructor.
pub struct ToolboxButtonProps {
    pub caption: Box<dyn SceneList>,
    pub variant: ButtonVariant,
}

impl Default for ToolboxButtonProps {
    fn default() -> Self {
        Self {
            caption: Box::new(bsn_list!()),
            variant: ButtonVariant::default(),
        }
    }
}

pub type ButtonProps = ToolboxButtonProps;

/// The Button Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ToolboxButtonProps)]
#[reflect(Component, Default)]
pub struct ToolboxButton;

pub type Button = ToolboxButton;

impl ToolboxButton {
    pub fn scene(props: ToolboxButtonProps) -> impl Scene {
        let initial_bg = match props.variant {
            ButtonVariant::Normal => palette::DEEP_SLATE_2,
            ButtonVariant::Primary => palette::ELECTRIC_BLUE,
            ButtonVariant::Plain => Color::NONE,
        };

        bsn! {
            Node {
                height: px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(12.0)),
                border_radius: BorderRadius::all(px(6.0)),
            }
            BevyButton
            ToolboxButton
            template_value(props.variant)
            Hovered
            BackgroundColor(initial_bg)
            Children [
                {props.caption}
            ]
        }
    }
}

/// Tool button scene component: a smaller button for embedding in panel headers.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ToolboxButtonProps)]
#[reflect(Component, Default)]
pub struct ToolboxToolButton;

pub type ToolButton = ToolboxToolButton;

impl ToolboxToolButton {
    pub fn scene(props: ToolboxButtonProps) -> impl Scene {
        bsn! {
            @ToolboxButton {
                @caption: {props.caption},
                @variant: {props.variant},
            }
            Node {
                padding: UiRect::horizontal(px(4.0)),
                min_width: px(24.0),
            }
        }
    }
}

type ButtonVisualsQueryData<'a> = (
    &'a ButtonVariant,
    &'a Hovered,
    Has<Pressed>,
    Mut<'a, BackgroundColor>,
);
type ButtonVisualsFilter = (With<ToolboxButton>, Or<(Changed<Hovered>, Added<Pressed>)>);

/// Reactive system that updates background colors when hovered or pressed.
fn update_button_visuals(mut q_buttons: Query<ButtonVisualsQueryData, ButtonVisualsFilter>) {
    for (variant, hovered, pressed, mut bg) in q_buttons.iter_mut() {
        let new_color = match (variant, pressed, hovered.0) {
            // Normal variant
            (ButtonVariant::Normal, true, _) => palette::DEEP_SLATE_1,
            (ButtonVariant::Normal, false, true) => palette::STEEL_SLATE,
            (ButtonVariant::Normal, false, false) => palette::DEEP_SLATE_2,

            // Primary variant
            (ButtonVariant::Primary, true, _) => palette::ELECTRIC_BLUE_PRESSED,
            (ButtonVariant::Primary, false, true) => palette::ELECTRIC_BLUE_HOVER,
            (ButtonVariant::Primary, false, false) => palette::ELECTRIC_BLUE,

            // Plain variant
            (ButtonVariant::Plain, true, _) => Color::srgba(1.0, 1.0, 1.0, 0.15),
            (ButtonVariant::Plain, false, true) => Color::srgba(1.0, 1.0, 1.0, 0.08),
            (ButtonVariant::Plain, false, false) => Color::NONE,
        };

        bg.0 = new_color;
    }
}

/// Plugin registering reactive styling systems for buttons.
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_button_visuals);
    }
}
