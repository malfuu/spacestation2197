//! Main menu of the game, including UI and transfering states.
mod editor_menu;
mod join_menu;

use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::base::{
    menus::main_menu::{
        editor_menu::{load_available_maps, ui_editor_menu},
        join_menu::ui_join_menu,
    },
    session::JoinGame,
    states::AppState,
};

pub(super) struct ClientMainMenuPlugin;

impl Plugin for ClientMainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MainMenuState>()
            .add_systems(
                EguiPrimaryContextPass,
                ui_main_menu.run_if(in_state(MainMenuState::Main)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                ui_join_menu.run_if(in_state(MainMenuState::Join)),
            )
            // editor menu related.
            .add_systems(OnEnter(MainMenuState::Editor), load_available_maps)
            .add_systems(
                EguiPrimaryContextPass,
                ui_editor_menu.run_if(in_state(MainMenuState::Editor)),
            );
    }
}

#[derive(SubStates, PartialEq, Eq, Hash, Default, Debug, Clone)]
#[source(AppState = AppState::Menu)]
enum MainMenuState {
    #[default]
    Main,
    Join,
    Editor,
}

fn ui_main_menu(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    egui::Window::new("Space Station 2197")
        .collapsible(false)
        .resizable(false)
        .max_width(160.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Main Menu");
            });

            if ui.button("Join").clicked() {
                commands.set_state(MainMenuState::Join);
            }

            if ui.button("Join Local").clicked() {
                commands.trigger(JoinGame::default());
            }

            if ui.button("Editor").clicked() {
                commands.set_state(MainMenuState::Editor);
            }
            ui.add_enabled(false, egui::Button::new("Options"));

            if ui.button("Exit").clicked() {
                commands.write_message(AppExit::Success);
            }
        });

    Ok(())
}
