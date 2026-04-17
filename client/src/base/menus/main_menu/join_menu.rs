use std::net::IpAddr;

use bevy::prelude::*;
use bevy_egui::prelude::*;

use shared::defines::DEFAULT_LISTEN_PORT;

use crate::base::{menus::main_menu::MainMenuState, session::JoinGame};

pub(super) fn ui_join_menu(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut input_host: Local<String>,
    mut input_port: Local<String>,
    mut message: Local<Option<String>>,
) -> Result {
    egui::Window::new("Join Menu")
        .collapsible(false)
        .resizable(false)
        .max_width(160.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut()?, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                commands.set_state(MainMenuState::Main);
            }

            ui.label("Host:");
            ui.text_edit_singleline(&mut *input_host);

            ui.label("Port:");
            ui.text_edit_singleline(&mut *input_port);

            if ui.button("Join Game").clicked() {
                let Ok(address) = input_host.parse::<IpAddr>() else {
                    *message = Some("Invalid host address.".to_string());
                    return;
                };

                let port = if input_port.trim().is_empty() {
                    DEFAULT_LISTEN_PORT
                } else {
                    match input_port.parse::<u16>() {
                        Ok(p) => p,
                        Err(_) => {
                            *message = Some("Invalid port.".to_string());
                            return;
                        }
                    }
                };

                *message = None;

                input_host.clear();
                input_port.clear();

                commands.trigger(JoinGame {
                    address,
                    port,
                    password: None,
                });
            }

            if let Some(msg) = &*message {
                ui.label(msg);
            }
        });

    Ok(())
}
