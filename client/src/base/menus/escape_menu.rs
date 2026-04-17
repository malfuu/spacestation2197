//! Escape menu of the game, principally when in game.
use bevy::prelude::*;
use bevy_egui::prelude::*;
use shared::game::ghost::GhostInput;

use crate::base::{session::Disconnect, states::AppState, windows::WindowStack};

pub const MENU_ESCAPE: &str = "escape";

pub(super) struct ClientEscapeMenuPlugin;

impl Plugin for ClientEscapeMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_escape_window);
    }
}

fn register_escape_window(mut commands: Commands, mut stack: ResMut<WindowStack>) {
    let id = commands.register_system(ui_escape_menu);
    stack.register(MENU_ESCAPE, id);
}

fn ui_escape_menu(
    mut contexts: EguiContexts,
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut stack: ResMut<WindowStack>,
) -> Result {
    let mut is_open = true;
    let mut in_menu = false;

    let response = egui::Window::new("Escape Menu")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .max_width(160.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut()?, |ui| match state.get() {
            AppState::Menu => {
                // dirty hack cuz escape menu shouldnt be in main menu
                in_menu = true;
            }
            AppState::InGame => {
                if ui.button("Ghost").clicked() {
                    commands.write_message(GhostInput);
                }

                if ui.button("Disconnect").clicked() {
                    commands.trigger(Disconnect);
                }
            }
            AppState::Editor => {
                if ui.button("Exit").clicked() {
                    commands.set_state(AppState::Menu);
                }
            }
        });

    if !is_open || in_menu {
        stack.close(MENU_ESCAPE);
    }

    Ok(())
}
