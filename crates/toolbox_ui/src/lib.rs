//! Toolbox UI, a set of UI components and systems for use in the game.
//! Stylized and responsive, built on `bevy_ui` using Feathers (`bevy_feathers`) as a base of work and inspiration.

use bevy::prelude::*;

pub mod constants;
pub mod containers;
pub mod controls;
pub mod cursor;
pub mod dark_theme;
pub mod display;
pub mod focus;
pub mod font_styles;
pub mod palette;
pub mod rounded_corners;
pub mod theme;

pub mod prelude;

pub struct ToolboxUiPlugin;

impl Plugin for ToolboxUiPlugin {
    fn build(&self, _app: &mut App) {
    }
}
