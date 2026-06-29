//! Toolbox UI, a set of UI components and systems for use in the game.
//! Built on `bevy_ui`, it's analogous to `bevy_feathers` as a base of work and inspiration.

use bevy::prelude::*;

pub mod constants;
pub mod containers;
pub mod controls;
pub mod cursor;
pub mod display;
pub mod focus;
pub mod font_styles;
pub mod palette;
pub mod rounded_corners;
pub mod theme;
pub mod tokens;

pub mod prelude;

pub struct ToolboxUiPlugin;

impl Plugin for ToolboxUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            theme::ThemePlugin,
            controls::button::ButtonPlugin,
            controls::checkbox::CheckboxPlugin,
            controls::disclosure_toggle::DisclosureTogglePlugin,
            controls::listview::ListViewPlugin,
            controls::menu::MenuPlugin,
            controls::radio::RadioPlugin,
            controls::scrollbar::ScrollbarPlugin,
            controls::slider::SliderPlugin,
            controls::text_input::TextInputPlugin,
            controls::toggle_switch::ToggleSwitchPlugin,
        ));
    }
}
