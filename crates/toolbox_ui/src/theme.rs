use bevy::ecs::lifecycle::Insert;
use bevy::ecs::observer::On;
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

/// The unified user interface theme resource.
#[derive(Resource, Reflect, Debug)]
pub struct UiTheme {
    pub colors: HashMap<ThemeToken, Color>,
}

impl Default for UiTheme {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert(tokens::WINDOW_BG, Color::NONE);
        colors.insert(tokens::FOCUS_RING, palette::CYAN_GLOW);
        colors.insert(tokens::TEXT_MAIN, palette::TEXT_PRIMARY);
        colors.insert(tokens::TEXT_DIM, palette::TEXT_SECONDARY);

        // Buttons (Normal)
        colors.insert(tokens::BUTTON_BG, palette::DEEP_SLATE_2);
        colors.insert(tokens::BUTTON_BG_HOVER, palette::STEEL_SLATE);
        colors.insert(tokens::BUTTON_BG_PRESSED, palette::DEEP_SLATE_1);
        colors.insert(tokens::BUTTON_TEXT, palette::TEXT_PRIMARY);

        // Buttons (Primary Deep Blue)
        colors.insert(tokens::BUTTON_PRIMARY_BG, palette::ELECTRIC_BLUE);
        colors.insert(
            tokens::BUTTON_PRIMARY_BG_HOVER,
            palette::ELECTRIC_BLUE_HOVER,
        );
        colors.insert(
            tokens::BUTTON_PRIMARY_BG_PRESSED,
            palette::ELECTRIC_BLUE_PRESSED,
        );
        colors.insert(tokens::BUTTON_PRIMARY_TEXT, palette::TEXT_PRIMARY);

        // Buttons (Plain)
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
        colors.insert(tokens::SLIDER_BG, palette::DEEP_SLATE_1);
        colors.insert(tokens::SLIDER_BAR, palette::ELECTRIC_BLUE);
        colors.insert(tokens::SLIDER_TEXT, palette::TEXT_PRIMARY);

        // Scrollbar
        colors.insert(tokens::SCROLLBAR_BG, palette::DEEP_SLATE_0);
        colors.insert(tokens::SCROLLBAR_THUMB, palette::STEEL_SLATE);
        colors.insert(tokens::SCROLLBAR_THUMB_HOVER, palette::ELECTRIC_BLUE);

        // Checkbox
        colors.insert(tokens::CHECKBOX_BG, palette::DEEP_SLATE_1);
        colors.insert(tokens::CHECKBOX_BORDER, palette::BORDER_STEEL);
        colors.insert(tokens::CHECKBOX_MARK, palette::ELECTRIC_BLUE);
        colors.insert(tokens::CHECKBOX_TEXT, palette::TEXT_PRIMARY);

        // Radio
        colors.insert(tokens::RADIO_BORDER, palette::BORDER_STEEL);
        colors.insert(tokens::RADIO_MARK, palette::ELECTRIC_BLUE);
        colors.insert(tokens::RADIO_TEXT, palette::TEXT_PRIMARY);

        // Switch
        colors.insert(tokens::SWITCH_BG, palette::DEEP_SLATE_1);
        colors.insert(tokens::SWITCH_BORDER, palette::BORDER_STEEL);
        colors.insert(tokens::SWITCH_SLIDE_BG, palette::TEXT_SECONDARY);
        colors.insert(tokens::SWITCH_SLIDE_BG_CHECKED, palette::ELECTRIC_BLUE);

        // Menus
        colors.insert(tokens::MENU_BG, palette::DEEP_SLATE_1);
        colors.insert(tokens::MENU_BORDER, palette::BORDER_STEEL);
        colors.insert(tokens::MENUITEM_BG_HOVER, palette::DEEP_SLATE_2);
        colors.insert(tokens::MENUITEM_BG_PRESSED, palette::STEEL_SLATE);
        colors.insert(tokens::MENUITEM_TEXT, palette::TEXT_PRIMARY);

        // Pane / Subpane / Group
        colors.insert(tokens::PANE_HEADER_BG, palette::STEEL_SLATE);
        colors.insert(tokens::PANE_HEADER_BORDER, palette::BORDER_STEEL);
        colors.insert(tokens::PANE_HEADER_DIVIDER, palette::BORDER_STEEL);
        colors.insert(tokens::PANE_BODY_BG, palette::DEEP_SLATE_1);

        colors.insert(tokens::SUBPANE_HEADER_BG, palette::DEEP_SLATE_2);
        colors.insert(tokens::SUBPANE_HEADER_BORDER, palette::BORDER_STEEL);
        colors.insert(tokens::SUBPANE_BODY_BG, palette::DEEP_SLATE_0);

        colors.insert(tokens::GROUP_HEADER_BG, palette::STEEL_SLATE);
        colors.insert(tokens::GROUP_HEADER_BORDER, palette::BORDER_STEEL);
        colors.insert(tokens::GROUP_BODY_BG, palette::DEEP_SLATE_1);

        // Listview
        colors.insert(tokens::LISTROW_BG, Color::NONE);
        colors.insert(tokens::LISTROW_BG_HOVER, palette::DEEP_SLATE_2);
        colors.insert(tokens::LISTROW_BG_SELECTED, palette::STEEL_SLATE);
        colors.insert(tokens::LISTROW_TEXT, palette::TEXT_PRIMARY);

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
#[derive(Component, Clone, Default, Reflect)]
#[require(BackgroundColor)]
#[reflect(Component, Default)]
pub struct ThemeBackgroundColor(pub ThemeToken);

/// Component setting entity border color from theme token.
#[derive(Component, Clone, Default, Reflect)]
#[require(BorderColor)]
#[reflect(Component, Default)]
pub struct ThemeBorderColor(pub ThemeToken);

/// Component setting text color from theme token.
#[derive(Component, Clone, Default, Reflect)]
#[require(TextColor)]
#[reflect(Component, Default)]
pub struct ThemeTextColor(pub ThemeToken);

fn update_themed_components(
    theme: Res<UiTheme>,
    mut q_bg: Query<(&ThemeBackgroundColor, &mut BackgroundColor), Changed<ThemeBackgroundColor>>,
    mut q_border: Query<(&ThemeBorderColor, &mut BorderColor), Changed<ThemeBorderColor>>,
    mut q_text: Query<(&ThemeTextColor, &mut TextColor), Changed<ThemeTextColor>>,
) {
    if !theme.is_changed() {
        return;
    }
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

fn on_insert_bg(
    insert: On<Insert, ThemeBackgroundColor>,
    mut q_bg: Query<(&ThemeBackgroundColor, &mut BackgroundColor)>,
    theme: Res<UiTheme>,
) {
    let Ok((token, mut bg)) = q_bg.get_mut(insert.entity) else {
        return;
    };
    bg.0 = theme.color(&token.0);
}

fn on_insert_border(
    insert: On<Insert, ThemeBorderColor>,
    mut q_border: Query<(&ThemeBorderColor, &mut BorderColor)>,
    theme: Res<UiTheme>,
) {
    let Ok((token, mut border)) = q_border.get_mut(insert.entity) else {
        return;
    };
    *border = BorderColor::all(theme.color(&token.0));
}

fn on_insert_text(
    insert: On<Insert, ThemeTextColor>,
    mut q_text: Query<(&ThemeTextColor, &mut TextColor)>,
    theme: Res<UiTheme>,
) {
    let Ok((token, mut text)) = q_text.get_mut(insert.entity) else {
        return;
    };
    text.0 = theme.color(&token.0);
}

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiTheme>()
            .add_systems(PreUpdate, update_themed_components)
            .add_observer(on_insert_bg)
            .add_observer(on_insert_border)
            .add_observer(on_insert_text);
    }
}
