use bevy::prelude::*;
use bevy_egui::prelude::*;
use bevy_renet::RenetClient;

use crate::debug_tools::{AppDebugOptionExt, option_enabled};

const DEBUG_OPTION_NETWORK: &str = "network";

pub(super) struct DebugNetworkingPlugin;

impl Plugin for DebugNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.register_debug_option(DEBUG_OPTION_NETWORK).add_systems(
            EguiPrimaryContextPass,
            ui_debug_network.run_if(option_enabled(DEBUG_OPTION_NETWORK)),
        );
    }
}

fn ui_debug_network(mut contexts: EguiContexts, client: Option<Res<RenetClient>>) -> Result {
    egui::Window::new("Network Debug")
        .default_width(200.0)
        .show(contexts.ctx_mut()?, |ui| {
            let Some(client) = client else {
                ui.label("Disconnected");
                return;
            };

            let info = client.network_info();

            let rtt = format!("{:.2} ms", info.rtt / 1000.0);
            let packet_loss = format!("{:.2}%", info.packet_loss * 100.0);
            let sent_kb = format!("{:.2} KiB/s", info.bytes_sent_per_second / 1024.0);
            let recv_kb = format!("{:.2} KiB/s", info.bytes_received_per_second / 1024.0);

            let loss_color = if info.packet_loss > 0.15 {
                Some(egui::Color32::RED)
            } else if info.packet_loss > 0.05 {
                Some(egui::Color32::YELLOW)
            } else {
                None
            };

            egui::Grid::new("network_info_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("RTT:");
                    ui.label(rtt);
                    ui.end_row();

                    ui.label("Packet Loss:");
                    if let Some(color) = loss_color {
                        ui.colored_label(color, packet_loss);
                    } else {
                        ui.label(packet_loss);
                    }
                    ui.end_row();

                    ui.label("Sent:");
                    ui.label(sent_kb);
                    ui.end_row();

                    ui.label("Received:");
                    ui.label(recv_kb);
                    ui.end_row();
                });
        });

    Ok(())
}
