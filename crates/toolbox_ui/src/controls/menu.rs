use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{
    MenuButton as BevyMenuButton, MenuItem as BevyMenuItem, MenuPopup as BevyMenuPopup,
};

/// Top-level menu container.
#[derive(SceneComponent, Clone, Default, Reflect)]
#[reflect(Component, Default)]
pub struct Menu;

impl Menu {
    pub fn scene() -> impl Scene {
        bsn! {
            Node {
                height: px(24.0),
                justify_content: JustifyContent::Stretch,
                align_items: AlignItems::Stretch,
            }
            Menu
        }
    }
}

/// Menu button properties.
pub struct MenuButtonProps {
    pub caption: Box<dyn SceneList>,
}

impl Default for MenuButtonProps {
    fn default() -> Self {
        Self {
            caption: Box::new(bsn_list!()),
        }
    }
}

/// Menu button component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(MenuButtonProps)]
#[reflect(Component, Default)]
pub struct MenuButton;

impl MenuButton {
    pub fn scene(props: MenuButtonProps) -> impl Scene {
        bsn! {
            Node {
                height: px(24.0),
                padding: UiRect::horizontal(px(8.0)),
                align_items: AlignItems::Center,
            }
            BevyMenuButton
            MenuButton
            Hovered
            BackgroundColor(Color::srgb(0.20, 0.20, 0.24))
            Children [
                {props.caption}
            ]
        }
    }
}

/// Menu popup component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct MenuPopup;

impl MenuPopup {
    pub fn scene() -> impl Scene {
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(px(1.0)),
                padding: UiRect::vertical(px(4.0)),
                border_radius: BorderRadius::all(px(4.0)),
            }
            MenuPopup
            BevyMenuPopup
            Visibility::Hidden
            BackgroundColor(Color::srgb(0.16, 0.16, 0.19))
            BorderColor::all(Color::srgb(0.30, 0.30, 0.35))
        }
    }
}

/// Menu item properties.
pub struct MenuItemProps {
    pub caption: Box<dyn SceneList>,
}

impl Default for MenuItemProps {
    fn default() -> Self {
        Self {
            caption: Box::new(bsn_list!()),
        }
    }
}

/// Menu item component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(MenuItemProps)]
#[reflect(Component, Default)]
pub struct MenuItem;

impl MenuItem {
    pub fn scene(props: MenuItemProps) -> impl Scene {
        bsn! {
            Node {
                height: px(24.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(8.0)),
            }
            MenuItem
            BevyMenuItem
            Hovered
            TextColor(Color::srgb(0.90, 0.90, 0.95))
            Children [
                {props.caption}
            ]
        }
    }
}

/// Menu divider component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct MenuDivider;

impl MenuDivider {
    pub fn scene() -> impl Scene {
        bsn! {
            Node {
                height: px(1.0),
                align_self: AlignSelf::Stretch,
                margin: UiRect::vertical(px(2.0)),
            }
            BackgroundColor(Color::srgb(0.30, 0.30, 0.35))
        }
    }
}

/// Plugin registering systems for menus.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, _app: &mut App) {}
}
