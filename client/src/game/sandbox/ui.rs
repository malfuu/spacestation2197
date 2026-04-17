use bevy::prelude::*;
use bevy_egui::prelude::*;

use common::PrototypeId;
use content::prelude::*;
use shared::{defines::PROTOTYPE_TYPE_TILE, game::atmos::PROTOTYPE_TYPE_MIXTURE};

use crate::{base::windows::WindowStack, game::sandbox::SandboxMode};

pub const MENU_SANDBOX: &str = "sandbox";

#[derive(Default, Debug, PartialEq)]
pub(super) enum SandboxMenuState {
    #[default]
    Entities,
    Tiles,
    Atmos,
}

pub(super) fn register_sandbox_window(mut commands: Commands, mut stack: ResMut<WindowStack>) {
    let id = commands.register_system(ui_sandbox_menu);
    stack.register(MENU_SANDBOX, id);
}

pub(super) fn ui_sandbox_menu(
    mut contexts: EguiContexts,
    mut stack: ResMut<WindowStack>,
    mut sandbox_mode: ResMut<SandboxMode>,
    mut current_tab: Local<SandboxMenuState>,
    registry: Res<Prototypes>,
) -> Result {
    let mut is_open = true;

    egui::Window::new("Sandbox Menu")
        .open(&mut is_open)
        .resizable(true)
        .default_height(640.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.horizontal(|ui| {
                let tabs = [
                    SandboxMenuState::Entities,
                    SandboxMenuState::Tiles,
                    SandboxMenuState::Atmos,
                ];

                for tab_type in tabs {
                    let label = format!("{:?}", tab_type);
                    if ui
                        .selectable_label(*current_tab == tab_type, label)
                        .clicked()
                    {
                        *current_tab = tab_type;
                        *sandbox_mode = SandboxMode::Nothing;
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| match *current_tab {
                SandboxMenuState::Entities => {
                    render_catalog(
                        ui,
                        &registry,
                        &mut sandbox_mode,
                        PROTOTYPE_CATEGORY_ENTITY,
                        "Delete Entity",
                        SandboxMode::DeleteEntity,
                        SandboxMode::PlaceEntity,
                    );
                }
                SandboxMenuState::Tiles => {
                    render_catalog(
                        ui,
                        &registry,
                        &mut sandbox_mode,
                        PROTOTYPE_TYPE_TILE,
                        "Delete Tile",
                        SandboxMode::DeleteTile,
                        SandboxMode::PlaceTile,
                    );
                }
                SandboxMenuState::Atmos => {
                    render_catalog(
                        ui,
                        &registry,
                        &mut sandbox_mode,
                        PROTOTYPE_TYPE_MIXTURE,
                        "Clear Mixture",
                        SandboxMode::ClearMixture,
                        SandboxMode::PlaceMixture,
                    );
                }
            });
        });

    if !is_open {
        stack.close(MENU_SANDBOX);
    }

    Ok(())
}

fn render_catalog<F>(
    ui: &mut egui::Ui,
    registry: &Prototypes,
    sandbox_mode: &mut SandboxMode,
    category_key: &str,
    delete_label: &str,
    delete_command: SandboxMode,
    build_enum: F,
) where
    F: Fn(PrototypeId) -> SandboxMode,
{
    let is_tool_active = *sandbox_mode == delete_command;

    if ui
        .add(egui::Button::new(delete_label).selected(is_tool_active))
        .clicked()
    {
        *sandbox_mode = if is_tool_active {
            SandboxMode::Nothing
        } else {
            delete_command
        };
    }

    ui.separator();

    for entry in registry.iter_for_category_entries(category_key) {
        let id = entry;
        let target_mode = build_enum(id.clone());
        let is_selected = *sandbox_mode == target_mode;

        if ui.selectable_label(is_selected, id.to_string()).clicked() {
            *sandbox_mode = if is_selected {
                SandboxMode::Nothing
            } else {
                target_mode
            };
        }
    }
}
