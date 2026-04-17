use std::{fs, path::Path};

use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::{base::menus::main_menu::MainMenuState, editor::LoadEditor};

type MapId = String;

#[derive(Resource, Deref, DerefMut)]
pub(super) struct AvailableMaps(Vec<MapId>);

#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct SelectedMap(Option<MapId>);

pub(super) fn load_available_maps(mut commands: Commands) {
    let path = Path::new("assets/grids");

    let maps = fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.extension()?.to_str()? == "ron" {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    commands.insert_resource(SelectedMap::default());
    commands.insert_resource(AvailableMaps(maps));
}

pub(super) fn ui_editor_menu(
    mut contexts: EguiContexts,
    mut commands: Commands,
    maps: Res<AvailableMaps>,
    mut selected_map: ResMut<SelectedMap>,
) -> Result {
    egui::Window::new("Editor Menu")
        .collapsible(false)
        .resizable(false)
        .max_width(160.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut()?, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                commands.set_state(MainMenuState::Main);
            }

            if maps.is_empty() {
                ui.label("No maps available");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for map in maps.iter() {
                            let is_selected = selected_map.0.as_ref() == Some(map);

                            if ui.selectable_label(is_selected, map).clicked() {
                                **selected_map = Some(map.clone());
                            }
                        }
                    });
            }

            ui.separator();

            ui.horizontal(|ui| {
                let is_map_selected = selected_map.is_some();

                if ui
                    .add_enabled(is_map_selected, egui::Button::new("Load"))
                    .clicked()
                {
                    commands.trigger(LoadEditor {
                        grid_name: selected_map.0.clone(),
                    });
                }

                if ui.button("New").clicked() {
                    commands.trigger(LoadEditor { grid_name: None });
                }
            });
        });
    Ok(())
}
