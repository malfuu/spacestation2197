use bevy::prelude::*;
use std::collections::HashMap;

use crate::palette;
use crate::tokens;

/// A design token for the theme serving as the key for properties.
#[derive(Clone, PartialEq, Eq, Hash, Reflect, Default, Debug)]
pub struct ThemeToken(&'static str);

impl ThemeToken {
    pub const fn new_static(text: &'static str) -> Self {
        Self(text)
    }
}

impl std::fmt::Display for ThemeToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The single, unified user interface theme resource.
#[derive(Resource, Reflect, Debug)]
pub struct UiTheme {
    pub colors: HashMap<ThemeToken, Color>,
}

impl Default for UiTheme {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert(tokens::WINDOW_BG, palette::GRAY_0);
        colors.insert(tokens::FOCUS_RING, palette::ACCENT);
        colors.insert(tokens::TEXT_MAIN, palette::LIGHT_GRAY_1);
        colors.insert(tokens::TEXT_DIM, palette::LIGHT_GRAY_2);

        // Buttons
        colors.insert(tokens::BUTTON_BG, palette::GRAY_3);
        colors.insert(tokens::BUTTON_BG_HOVER, palette::GRAY_4);
        colors.insert(tokens::BUTTON_BG_PRESSED, palette::GRAY_2);
        colors.insert(tokens::BUTTON_TEXT, palette::WHITE);

        colors.insert(tokens::BUTTON_PRIMARY_BG, palette::ACCENT);
        colors.insert(
            tokens::BUTTON_PRIMARY_BG_HOVER,
            Color::srgb(0.20, 0.55, 0.95),
        );
        colors.insert(
            tokens::BUTTON_PRIMARY_BG_PRESSED,
            Color::srgb(0.10, 0.35, 0.70),
        );
        colors.insert(tokens::BUTTON_PRIMARY_TEXT, palette::WHITE);

        colors.insert(tokens::BUTTON_PLAIN_BG, Color::NONE);
        colors.insert(
            tokens::BUTTON_PLAIN_BG_HOVER,
            Color::srgba(1.0, 1.0, 1.0, 0.08),
        );
        colors.insert(
            tokens::BUTTON_PLAIN_BG_PRESSED,
            Color::srgba(1.0, 1.0, 1.0, 0.15),
        );

        // Slider
        colors.insert(tokens::SLIDER_BG, palette::GRAY_2);
        colors.insert(tokens::SLIDER_BAR, palette::ACCENT);
        colors.insert(tokens::SLIDER_TEXT, palette::WHITE);

        // Scrollbar
        colors.insert(tokens::SCROLLBAR_BG, palette::GRAY_0);
        colors.insert(tokens::SCROLLBAR_THUMB, palette::GRAY_4);
        colors.insert(tokens::SCROLLBAR_THUMB_HOVER, palette::ACCENT);

        // Checkbox
        colors.insert(tokens::CHECKBOX_BG, palette::GRAY_2);
        colors.insert(tokens::CHECKBOX_BORDER, palette::WARM_GRAY_1);
        colors.insert(tokens::CHECKBOX_MARK, palette::WHITE);
        colors.insert(tokens::CHECKBOX_TEXT, palette::LIGHT_GRAY_1);

        // Radio
        colors.insert(tokens::RADIO_BORDER, palette::WARM_GRAY_1);
        colors.insert(tokens::RADIO_MARK, palette::ACCENT);
        colors.insert(tokens::RADIO_TEXT, palette::LIGHT_GRAY_1);

        // Switch
        colors.insert(tokens::SWITCH_BG, palette::GRAY_2);
        colors.insert(tokens::SWITCH_BORDER, palette::WARM_GRAY_1);
        colors.insert(tokens::SWITCH_SLIDE_BG, palette::LIGHT_GRAY_2);
        colors.insert(tokens::SWITCH_SLIDE_BG_CHECKED, palette::ACCENT);

        // Menus
        colors.insert(tokens::MENU_BG, palette::GRAY_1);
        colors.insert(tokens::MENU_BORDER, palette::WARM_GRAY_1);
        colors.insert(tokens::MENUITEM_BG_HOVER, palette::GRAY_3);
        colors.insert(tokens::MENUITEM_BG_PRESSED, palette::GRAY_4);
        colors.insert(tokens::MENUITEM_TEXT, palette::LIGHT_GRAY_1);

        // Pane / Subpane / Group
        colors.insert(tokens::PANE_HEADER_BG, Color::srgb(0.20, 0.20, 0.23));
        colors.insert(tokens::PANE_HEADER_BORDER, Color::srgb(0.30, 0.30, 0.34));
        colors.insert(tokens::PANE_HEADER_DIVIDER, Color::srgb(0.35, 0.35, 0.40));
        colors.insert(tokens::PANE_BODY_BG, Color::srgb(0.15, 0.15, 0.18));

        colors.insert(tokens::SUBPANE_HEADER_BG, Color::srgb(0.18, 0.18, 0.21));
        colors.insert(tokens::SUBPANE_HEADER_BORDER, Color::srgb(0.28, 0.28, 0.32));
        colors.insert(tokens::SUBPANE_BODY_BG, Color::srgb(0.13, 0.13, 0.16));

        colors.insert(tokens::GROUP_HEADER_BG, Color::srgb(0.22, 0.22, 0.25));
        colors.insert(tokens::GROUP_HEADER_BORDER, Color::srgb(0.32, 0.32, 0.36));
        colors.insert(tokens::GROUP_BODY_BG, Color::srgb(0.16, 0.16, 0.19));

        // Listview
        colors.insert(tokens::LISTROW_BG, Color::NONE);
        colors.insert(tokens::LISTROW_BG_HOVER, palette::GRAY_3);
        colors.insert(tokens::LISTROW_BG_SELECTED, palette::GRAY_4);
        colors.insert(tokens::LISTROW_TEXT, palette::LIGHT_GRAY_1);

        Self { colors }
    }
}

impl UiTheme {
    pub fn color(&self, token: &ThemeToken) -> Color {
        self.colors
            .get(token)
            .copied()
            .unwrap_or(Color::srgb(1.0, 0.0, 1.0))
    }
}

/// Component setting entity background color from theme token.
#[derive(Component, Clone, Debug)]
pub struct ThemeBackgroundColor(pub ThemeToken);

/// Component setting entity border color from theme token.
#[derive(Component, Clone, Debug)]
pub struct ThemeBorderColor(pub ThemeToken);

/// Component setting text color from theme token.
#[derive(Component, Clone, Debug)]
pub struct ThemeTextColor(pub ThemeToken);

fn update_themed_components(
    theme: Res<UiTheme>,
    mut q_bg: Query<(&ThemeBackgroundColor, &mut BackgroundColor), Changed<ThemeBackgroundColor>>,
    mut q_border: Query<(&ThemeBorderColor, &mut BorderColor), Changed<ThemeBorderColor>>,
    mut q_text: Query<(&ThemeTextColor, &mut TextColor), Changed<ThemeTextColor>>,
) {
    if theme.is_changed() {
        for (token, mut bg) in q_bg.iter_mut() {
            bg.0 = theme.color(&token.0);
        }
        for (token, mut border) in q_border.iter_mut() {
            *border = BorderColor::all(theme.color(&token.0));
        }
        for (token, mut text) in q_text.iter_mut() {
            text.0 = theme.color(&token.0);
        }
    }
}

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiTheme>()
            .add_systems(PreUpdate, update_themed_components);
    }
}
