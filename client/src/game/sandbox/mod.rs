pub mod ui;

use bevy::prelude::*;
use common::PrototypeId;
use shared::game::{placement::Placement, sandbox::SandboxCommands};

use crate::{
    base::{input::ExtraInputs, windows::WindowCommands},
    game::sandbox::ui::{MENU_SANDBOX, register_sandbox_window},
};

pub(super) struct ClientSandboxPlugin;

impl Plugin for ClientSandboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SandboxMode>()
            // .init_resource::<SandboxMenuState>()
            .add_systems(Startup, register_sandbox_window)
            .add_systems(Update, open_sandbox)
            .add_systems(Update, on_click);
    }
}

#[derive(Resource, Default, PartialEq, Clone)]
enum SandboxMode {
    #[default]
    Nothing,

    PlaceEntity(PrototypeId),
    PlaceTile(PrototypeId),
    PlaceMixture(PrototypeId),

    DeleteEntity,
    DeleteTile,
    ClearMixture,
}

fn on_click(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut sandbox_mode: ResMut<SandboxMode>,
    extra_inputs: Res<ExtraInputs>,
) {
    if mouse_buttons.just_pressed(MouseButton::Right) {
        *sandbox_mode = SandboxMode::Nothing;
    }

    let Some(mouse_positions) = extra_inputs.mouse_positions() else {
        return;
    };

    if mouse_buttons.just_pressed(MouseButton::Left) {
        match sandbox_mode.clone() {
            SandboxMode::PlaceEntity(id) => {
                commands.write_message(SandboxCommands::Place(Placement::Entity {
                    entity: id,
                    position: mouse_positions.ground_plane_position.xz(),
                }));
            }
            SandboxMode::PlaceTile(id) => {
                commands.write_message(SandboxCommands::Place(Placement::Tile {
                    tile: id,
                    start: mouse_positions.tile_position,
                    end: mouse_positions.tile_position,
                }));
            }
            SandboxMode::PlaceMixture(id) => {
                commands.write_message(SandboxCommands::SetMixture(
                    Some(id),
                    mouse_positions.tile_position,
                ));
            }
            SandboxMode::DeleteTile => {
                commands.write_message(SandboxCommands::EraseTile(mouse_positions.tile_position));
            }
            SandboxMode::ClearMixture => {
                commands.write_message(SandboxCommands::SetMixture(
                    None,
                    mouse_positions.tile_position,
                ));
            }
            SandboxMode::DeleteEntity => {
                if let Some(entity) = extra_inputs.hovering() {
                    commands.write_message(SandboxCommands::EraseEntity(entity));
                }
            }
            SandboxMode::Nothing => {}
        }
    }
}

fn open_sandbox(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F6) {
        commands.toggle_window(MENU_SANDBOX);
    }
}
