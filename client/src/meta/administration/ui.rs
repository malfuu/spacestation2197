use bevy::prelude::*;
use bevy_egui::prelude::*;
use bevy_replicon::shared::backend::connected_client::NetworkId;
use shared::{
    game::player::Player,
    meta::{administration::AdminCommandMessage, gamemode::Gamemode, player::PlayerName},
};

use crate::base::windows::{WindowCommands, WindowStack};

pub const MENU_ADMIN: &str = "administration";

pub(super) struct ClientAdministrationUiPlugin;

impl Plugin for ClientAdministrationUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AdminMenuState>()
            .add_systems(Startup, register_admin_window)
            .add_systems(Update, open_admin_menu);
    }
}

#[derive(Resource, Debug, Default, PartialEq, Eq, Clone, Copy)]
enum AdminMenuState {
    #[default]
    Server,
    Players,
    Round,
}

pub(super) fn register_admin_window(mut commands: Commands, mut stack: ResMut<WindowStack>) {
    let id = commands.register_system(ui_admin_menu);
    stack.register(MENU_ADMIN, id);
}

fn ui_admin_menu(
    mut contexts: EguiContexts,
    mut current_tab: ResMut<AdminMenuState>,
    mut stack: ResMut<WindowStack>,
    mut commands: Commands,
    players: Query<(&PlayerName, &NetworkId), With<Player>>,
    mut select_gamemode: Local<Gamemode>,
) -> Result {
    let mut is_open = true;

    egui::Window::new("Admin Menu")
        .open(&mut is_open)
        .resizable(true)
        .min_width(400.0)
        .show(contexts.ctx_mut().map_err(|_| "Error")?, |ui| {
            ui.horizontal(|ui| {
                let tabs = [
                    AdminMenuState::Server,
                    AdminMenuState::Players,
                    AdminMenuState::Round,
                ];

                for tab_type in tabs {
                    let label = format!("{:?}", tab_type);
                    let is_selected = *current_tab == tab_type;

                    if ui.selectable_label(is_selected, label).clicked() {
                        *current_tab = tab_type;
                    }
                }
            });

            ui.separator();

            match *current_tab {
                AdminMenuState::Server => ui_admin_server(ui, &mut commands),
                AdminMenuState::Players => ui_admin_players(ui, &mut commands, &players),
                AdminMenuState::Round => ui_admin_round(ui, &mut commands, &mut select_gamemode),
            }
        });

    if !is_open {
        stack.close(MENU_ADMIN);
    }

    Ok(())
}

fn open_admin_menu(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F8) {
        commands.toggle_window(MENU_ADMIN);
    }
}

fn ui_admin_server(ui: &mut egui::Ui, commands: &mut Commands) {
    ui.heading("Server Controls");
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label("TPS:");
        if ui.button("30").clicked() {
            commands.write_message(AdminCommandMessage::SetTps(30));
        }
        if ui.button("60").clicked() {
            commands.write_message(AdminCommandMessage::SetTps(60));
        }
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Atmos:");
        if ui.button("On").clicked() {
            commands.write_message(AdminCommandMessage::SetAtmos(true));
        }
        if ui.button("Off").clicked() {
            commands.write_message(AdminCommandMessage::SetAtmos(false));
        }
    });

    ui.horizontal(|ui| {
        ui.label("Physics:");
        if ui.button("On").clicked() {
            commands.write_message(AdminCommandMessage::SetPhysics(true));
        }
        if ui.button("Off").clicked() {
            commands.write_message(AdminCommandMessage::SetPhysics(false));
        }
    });

    ui.horizontal(|ui| {
        ui.label("Gameplay:");
        if ui.button("On").clicked() {
            commands.write_message(AdminCommandMessage::SetGameplay(true));
        }
        if ui.button("Off").clicked() {
            commands.write_message(AdminCommandMessage::SetGameplay(false));
        }
    });

    ui.horizontal(|ui| {
        ui.label("OOC:");
        if ui.button("On").clicked() {
            commands.write_message(AdminCommandMessage::SetOoc(true));
        }
        if ui.button("Off").clicked() {
            commands.write_message(AdminCommandMessage::SetOoc(false));
        }
    });
}

fn ui_admin_players(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    players: &Query<(&PlayerName, &NetworkId), With<Player>>,
) {
    ui.heading("Player Controls");
    ui.add_space(5.0);

    for (name, net_id) in players.iter() {
        ui.horizontal(|ui| {
            ui.label(name.get());

            if ui.button("BWOINK").clicked() {
                commands.write_message(AdminCommandMessage::AdminMessage(*net_id));
            }
            if ui.button("Kick").clicked() {
                commands.write_message(AdminCommandMessage::Kick(*net_id));
            }
            if ui.button("Ban").clicked() {
                commands.write_message(AdminCommandMessage::Kick(*net_id));
            }
            if ui.button("Respawn").clicked() {
                commands.write_message(AdminCommandMessage::Respawn(*net_id));
            }
        });
    }
}

fn ui_admin_round(ui: &mut egui::Ui, commands: &mut Commands, selected_gamemode: &mut Gamemode) {
    ui.heading("Round Controls");
    ui.add_space(5.0);

    if ui.button("Force Start Round").clicked() {
        commands.write_message(AdminCommandMessage::ForceStartRound);
    }

    if ui.button("Force End Round").clicked() {
        commands.write_message(AdminCommandMessage::ForceEndRound);
    }

    ui.horizontal(|ui| {
        ui.label("Gamemode:");

        egui::ComboBox::from_id_salt("gamemode_select")
            .selected_text(selected_gamemode.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    selected_gamemode,
                    Gamemode::Extended,
                    Gamemode::Extended.to_string(),
                );
                ui.selectable_value(
                    selected_gamemode,
                    Gamemode::Sandbox,
                    Gamemode::Sandbox.to_string(),
                );
                ui.selectable_value(
                    selected_gamemode,
                    Gamemode::Mafia,
                    Gamemode::Mafia.to_string(),
                );
            });

        if ui.button("Set").clicked() {
            commands.write_message(AdminCommandMessage::SetGamemode(selected_gamemode.clone()));
        }
    });

    if ui.button("SHUTDOWN SERVER").clicked() {
        commands.write_message(AdminCommandMessage::Shutdown);
    }
}
