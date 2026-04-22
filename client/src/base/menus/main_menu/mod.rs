//! Main menu of the game, including UI and transfering states.
mod editor_menu;
mod join_menu;

use bevy::prelude::*;
use bevy_egui::prelude::*;

use content::prelude::*;

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
            )
            // context hash text
            // NOTE: ApplyDeferred because entity needs to be present at show_content
            // and we do show_content because spawn_content_hash_text happens before
            // OnEnter(AppState::Menu)
            .add_systems(
                Startup,
                (
                    spawn_content_hash_text,
                    ApplyDeferred,
                    show_content_hash_text,
                )
                    .chain(),
            )
            .add_systems(
                PostStartup,
                update_content_hash_text.after(LoadContentSystems),
            )
            .add_systems(OnEnter(AppState::Menu), show_content_hash_text)
            .add_systems(OnExit(AppState::Menu), hide_content_hash_text);
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

#[derive(Component)]
struct ContentHashText;

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

fn spawn_content_hash_text(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(10.0),
            bottom: px(10.0),
            ..default()
        },
        Text("Content not loaded.".to_string()),
        Visibility::Hidden,
        ContentHashText,
    ));
}

fn update_content_hash_text(
    mut query: Query<&mut Text, With<ContentHashText>>,
    content_hash: Res<ContentHash>,
) {
    let mut text = query
        .single_mut()
        .expect("There should be a single context hash text.");
    *text = Text(format!("{}", content_hash.into_inner()));
}

fn show_content_hash_text(mut query: Query<&mut Visibility, With<ContentHashText>>) {
    if let Ok(mut visibility) = query.single_mut() {
        *visibility = Visibility::Inherited;
    }
}

fn hide_content_hash_text(mut query: Query<&mut Visibility, With<ContentHashText>>) {
    let mut visibility = query
        .single_mut()
        .expect("There should be a single context hash text.");
    *visibility = Visibility::Hidden;
}
