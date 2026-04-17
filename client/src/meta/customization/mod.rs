//! Player character & role customization!
use bevy::prelude::*;
use bevy_egui::prelude::*;

use shared::meta::customization::{CharacterSettings, PlayerSettings, SetCustomizationInput};

use crate::base::{session::ThisPlayer, windows::WindowStack};

pub const MENU_CUSTOMIZATION: &str = "customization";

pub(super) struct ClientCustomizationPlugin;

impl Plugin for ClientCustomizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_customization_window);
    }
}

fn register_customization_window(mut commands: Commands, mut stack: ResMut<WindowStack>) {
    let id = commands.register_system(ui_customization);
    stack.register(MENU_CUSTOMIZATION, id);
}

fn ui_customization(
    mut contexts: EguiContexts,
    mut stack: ResMut<WindowStack>,
    mut buffer: Local<Option<CharacterSettings>>,
    player: Single<&PlayerSettings, With<ThisPlayer>>,
    mut commands: Commands,
) -> Result {
    let mut is_open = true;

    let settings = buffer.get_or_insert_with(|| player.character.clone());

    egui::Window::new("Character Customization")
        .open(&mut is_open)
        .resizable(true)
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical(|ui| {
                ui.label("Character Name");
                ui.text_edit_singleline(&mut settings.name);

                ui.add_space(8.0);

                ui.label("Skin Color");

                let mut color_array = [
                    settings.skin_color.red,
                    settings.skin_color.green,
                    settings.skin_color.blue,
                ];

                if ui.color_edit_button_rgb(&mut color_array).changed() {
                    settings.skin_color.red = color_array[0];
                    settings.skin_color.green = color_array[1];
                    settings.skin_color.blue = color_array[2];
                }

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Randomize").clicked() {
                        *settings = CharacterSettings::random();
                    }

                    if ui.button("Save").clicked() {
                        commands.write_message(SetCustomizationInput(settings.clone()));
                    }
                });
            });
        });

    if !is_open {
        *buffer = None;
        stack.close(MENU_CUSTOMIZATION);
    }

    Ok(())
}
