use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::Button as BevyButton;

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
            ButtonVariant::Normal => Color::srgb(0.25, 0.25, 0.28),
            ButtonVariant::Primary => Color::srgb(0.15, 0.45, 0.85),
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

/// Reactive system that updates background colors when hovered or pressed.
fn update_button_visuals(
    mut q_buttons: Query<
        (&ButtonVariant, &Hovered, Has<Pressed>, &mut BackgroundColor),
        (With<ToolboxButton>, Or<(Changed<Hovered>, Added<Pressed>)>),
    >,
) {
    for (variant, hovered, pressed, mut bg) in q_buttons.iter_mut() {
        let new_color = match (variant, pressed, hovered.0) {
            // Normal variant
            (ButtonVariant::Normal, true, _) => Color::srgb(0.18, 0.18, 0.20),
            (ButtonVariant::Normal, false, true) => Color::srgb(0.32, 0.32, 0.36),
            (ButtonVariant::Normal, false, false) => Color::srgb(0.25, 0.25, 0.28),

            // Primary variant
            (ButtonVariant::Primary, true, _) => Color::srgb(0.10, 0.35, 0.70),
            (ButtonVariant::Primary, false, true) => Color::srgb(0.20, 0.55, 0.95),
            (ButtonVariant::Primary, false, false) => Color::srgb(0.15, 0.45, 0.85),

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
