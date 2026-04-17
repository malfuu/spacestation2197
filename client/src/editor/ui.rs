use core::fmt;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::{egui::Ui, prelude::*};

use content::prelude::*;
use shared::defines::PROTOTYPE_TYPE_TILE;

use crate::{
    base::states::AppState,
    editor::{EditorResource, SaveMap},
};

pub(super) struct ClientEditorUiPlugin;

impl Plugin for ClientEditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<EditorGizmos>()
            .init_resource::<EditorUiResource>()
            .add_systems(Startup, initialize_editor_gizmos)
            .add_systems(
                Update,
                (draw_editor_gizmos, update_editor_gizmos_config)
                    .run_if(in_state(AppState::Editor)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                ui_editor_palette.run_if(in_state(AppState::Editor)),
            );
    }
}

#[derive(Resource, Default, PartialEq)]
pub enum EditorUiResource {
    #[default]
    File,
    Tiles,
    Entities,
    Atmos,
}

impl fmt::Display for EditorResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorResource::None => write!(f, "None"),
            EditorResource::Entity(id) => write!(f, "Entity: {}", id),
            EditorResource::Tile(id) => write!(f, "Tile: {}", id),
            EditorResource::Atmos(id) => write!(f, "Atmos: {}", id),
            EditorResource::AtmosClear => write!(f, "Atmos Clear"),
        }
    }
}

fn ui_editor_palette(
    mut contexts: EguiContexts,
    mut commands: Commands,
    prototype_registry: Res<Prototypes>,
    mut ui_state: ResMut<EditorUiResource>,
    mut selection: ResMut<EditorResource>,
    mut save_name: Local<String>,
) {
    let ctx = contexts.ctx_mut().expect("Failed to get egui context");

    egui::Window::new("Editor Pick")
        .anchor(egui::Align2::RIGHT_TOP, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let tabs = [
                    ("File", EditorUiResource::File),
                    ("Tiles", EditorUiResource::Tiles),
                    ("Entities", EditorUiResource::Entities),
                    ("Atmos", EditorUiResource::Atmos),
                ];

                for (label, tab_type) in tabs {
                    if ui.selectable_label(*ui_state == tab_type, label).clicked() {
                        *ui_state = tab_type;
                        *selection = EditorResource::None;
                    }
                }

                ui.label(selection.to_string());
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(300.0)
                .show(ui, |ui| match *ui_state {
                    EditorUiResource::File => {
                        ui_editor_file(ui, &mut commands, &mut save_name);
                    }
                    EditorUiResource::Tiles => {
                        ui_editor_tile(ui, &prototype_registry, &mut selection);
                    }
                    EditorUiResource::Entities => {
                        ui_editor_entities(ui, &prototype_registry, &mut selection);
                    }
                    EditorUiResource::Atmos => {
                        // ui_editor_atmos(ui, &mixture_list, &mut selection);
                    }
                });

            if *selection != EditorResource::None {
                ui.separator();
                if ui.button("Clear Selection").clicked() {
                    *selection = EditorResource::None;
                }
            }
        });
}

fn ui_editor_file(ui: &mut Ui, commands: &mut Commands, save_name: &mut String) {
    ui.heading("File Operations");

    ui.add(egui::TextEdit::singleline(&mut *save_name).hint_text("Map name..."));
    ui.label("without .ron!");
    if ui.button("Save Map").clicked() {
        info!("Saving: {}", *save_name);

        commands.trigger(SaveMap {
            name: save_name.clone(),
        });

        save_name.clear();
    }
}

fn ui_editor_tile(ui: &mut Ui, prototype_registry: &Prototypes, state: &mut EditorResource) {
    ui.heading("Tiles");
    for prototype in prototype_registry.iter_for_category_entries(PROTOTYPE_TYPE_TILE) {
        if ui
            .selectable_label(
                *state == EditorResource::Tile(prototype.clone()),
                prototype.to_string(),
            )
            .clicked()
        {
            *state = EditorResource::Tile(prototype.clone());
        }
    }
}

fn ui_editor_entities(ui: &mut Ui, prototype_registry: &Prototypes, state: &mut EditorResource) {
    ui.heading("Entities");
    for prototype in prototype_registry.iter_for_category_entries(PROTOTYPE_CATEGORY_ENTITY) {
        if ui
            .selectable_label(
                *state == EditorResource::Entity(prototype.clone()),
                prototype.to_string(),
            )
            .clicked()
        {
            *state = EditorResource::Entity(prototype.clone());
        }
    }
}

#[derive(GizmoConfigGroup, Reflect, Default)]
struct EditorGizmos;

fn initialize_editor_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (gizmo_configs, _) = config_store.config_mut::<EditorGizmos>();

    gizmo_configs.line.width = 1.0;
}

fn draw_editor_gizmos(mut editor_gizmos: Gizmos<EditorGizmos>) {
    editor_gizmos.arrow(Vec3::ZERO, Vec3::X, Srgba::from_vec3(Vec3::X));
    editor_gizmos.arrow(Vec3::ZERO, Vec3::Y, Srgba::from_vec3(Vec3::Y));
    editor_gizmos.arrow(Vec3::ZERO, Vec3::Z, Srgba::from_vec3(Vec3::Z));

    // TODO: change this to infinite grid instead
    editor_gizmos.grid(
        Quat::from_rotation_x(PI / 2.),
        UVec2::splat(100),
        Vec2::new(1., 1.),
        LinearRgba::gray(0.55),
    );
}

fn update_editor_gizmos_config(mut editor_gizmos: Gizmos<EditorGizmos>) {}

fn on_editor_enter(mut camera: Single<&mut Transform, With<Camera>>) {}

fn on_editor_exit() {}
