//! In Game Map Editor related.
mod ui;

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use common::{EntityTag, PrototypeId, TileTag};
use content::entity::PrototypeEntityCommandsExt;
use grid::Grid;
use shared::game::{
    grid::GridCommandsExt,
    persistence::{MapInformation, load_grid},
};

use crate::{
    base::{
        camera::{CameraMode, CameraState},
        input::ExtraInputs,
        states::AppState,
    },
    editor::ui::ClientEditorUiPlugin,
};

const EDITOR_CAMERA_MOVESPEED: f32 = 5.0;

pub(super) struct ClientEditorPlugin;

impl Plugin for ClientEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientEditorUiPlugin)
            .init_resource::<EditorResource>()
            .add_systems(
                Update,
                (editor_camera_inputs, editor_edit_inputs).run_if(in_state(AppState::Editor)),
            )
            .add_systems(OnEnter(AppState::Editor), on_editor_enter)
            .add_systems(OnExit(AppState::Editor), on_editor_exit)
            .add_observer(on_load_editor)
            .add_observer(on_save_map)
            .add_observer(on_entity_add)
            .add_observer(on_grid_add);
    }
}

fn editor_edit_inputs(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    extra_inputs: Res<ExtraInputs>,
    mut editor: ResMut<EditorResource>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let Some(mouse_positions) = extra_inputs.mouse_positions() else {
            return;
        };

        match &*editor {
            EditorResource::Entity(prototype) => {
                commands.spawn_prototype(
                    prototype.clone(),
                    Transform::from_translation(mouse_positions.ground_plane_position),
                );
            }
            EditorResource::Tile(tile) => {
                commands.spawn_tile(tile.clone(), mouse_positions.tile_position);
            }
            EditorResource::Atmos(mixture) => {}
            EditorResource::AtmosClear => {}
            EditorResource::None => { /* NOTHING */ }
        }
    }

    if mouse_input.just_pressed(MouseButton::Right) {
        *editor = EditorResource::None;
    }
}

fn editor_camera_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera: ResMut<CameraState>,
    time: Res<Time>,
) {
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        dir += Vec2::NEG_Y;
    }

    if keys.pressed(KeyCode::KeyA) {
        dir += Vec2::NEG_X;
    }

    if keys.pressed(KeyCode::KeyS) {
        dir += Vec2::Y;
    }

    if keys.pressed(KeyCode::KeyD) {
        dir += Vec2::X;
    }

    if let CameraMode::Free = camera.mode {
        let rotated_dir = (Quat::from_rotation_y(camera.angle) * Vec3::new(dir.x, 0.0, dir.y)).xz();

        camera.position +=
            rotated_dir.normalize_or_zero() * EDITOR_CAMERA_MOVESPEED * time.delta_secs();
    } else {
        error!("camera mode not free on editor.");
    }
}

#[derive(Resource, Default, PartialEq)]
struct EditorData {
    grid_name: Option<String>, // im just using this to save the name while switching states.
}

#[derive(Resource, Default, PartialEq)]
enum EditorResource {
    #[default]
    None,
    Entity(PrototypeId),
    Tile(TileTag),
    Atmos(PrototypeId),
    AtmosClear,
}

/// Triggers a save
#[derive(Event, Debug)]
struct SaveMap {
    name: String,
}

fn on_entity_add(add: On<Add, EntityTag>, mut commands: Commands, state: Res<State<AppState>>) {
    if !matches!(*state.get(), AppState::Editor) {
        // until observer run conditions arrive
        return;
    }
    commands
        .entity(add.entity)
        .insert(DespawnOnExit(AppState::Editor));
}

fn on_grid_add(add: On<Add, Grid>, mut commands: Commands, state: Res<State<AppState>>) {
    if !matches!(*state.get(), AppState::Editor) {
        // until observer run conditions arrive
        return;
    }

    commands
        .entity(add.entity)
        .insert(DespawnOnExit(AppState::Editor));
}

fn on_save_map(on: On<SaveMap>, mut commands: Commands) {
    commands.run_system_cached_with(save_map, on.name.clone());
}

fn save_map(grid_name: In<String>, world: &mut World) {
    let Some(map) = MapInformation::from_world(&grid_name.0, world) else {
        return;
    };
    let Some(serialized) = map.serialize() else {
        error!("Failed to serialize map!");
        return;
    };

    let grids_dir = Path::new("assets/grids");
    let ron_path: PathBuf = grids_dir.join(format!("{}.ron", grid_name.0.clone()));

    let mut file = match fs::File::create(&ron_path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create file {:?}: {}", ron_path, e);
            return;
        }
    };

    if let Err(e) = file.write_all(serialized.as_bytes()) {
        error!("Failed to write to {:?}: {}", ron_path, e);
    }
}

/// Loads into editor state and also loads the map.
#[derive(Event, Debug)]
pub struct LoadEditor {
    /// none if new map
    pub grid_name: Option<String>,
}

fn on_load_editor(on: On<LoadEditor>, mut commands: Commands) {
    commands.set_state(AppState::Editor);
    commands.insert_resource(EditorData {
        grid_name: on.grid_name.clone(),
    });
}

fn on_editor_enter(mut commands: Commands, editor_data: Res<EditorData>) {
    if let Some(name) = &editor_data.grid_name {
        commands.run_system_cached_with(load_grid, name.clone());
    } else {
        commands.spawn((Grid::new(), DespawnOnExit(AppState::Editor)));
    }
}

fn on_editor_exit(mut commands: Commands) {
    commands.remove_resource::<EditorData>();
}
