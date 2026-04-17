use bevy::prelude::*;
use bevy_egui::{egui::Ui, prelude::*};

use shared::meta::{
    gamemode::Gamemode,
    manager::Manager,
    round::{JoinInput, Ready, ReadyInput, RoundState, StartRoundTimer},
};

use crate::{
    base::{session::ThisPlayer, windows::WindowStack},
    game::mind::MindState,
    meta::customization::MENU_CUSTOMIZATION,
};

const LOBBY_ART: &str = "images/lobby/test.png";

pub(super) struct ClientLobbyPlugin;

impl Plugin for ClientLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            ui_lobby.run_if(in_state(MindState::NotControlling)),
        )
        .add_systems(OnEnter(MindState::NotControlling), draw_lobby_art)
        .add_systems(OnExit(MindState::NotControlling), delete_lobby_art);
    }
}

#[derive(Component)]
struct LobbyArt;

fn ui_lobby(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mind_state: Option<Res<State<MindState>>>,
    manager: Single<(&RoundState, Option<&StartRoundTimer>, &Gamemode), With<Manager>>,
    mut stack: ResMut<WindowStack>,
    player_query: Single<Option<&Ready>, With<ThisPlayer>>,
) -> Result {
    let (round_state, round_timer_opt, gamemode) = manager.into_inner();

    egui::Window::new("Lobby")
        .collapsible(false)
        .resizable(false)
        .default_size([400.0, 300.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(format!("Gamemode: {gamemode}"));

                match *round_state {
                    RoundState::Starting => {
                        let time_left = round_timer_opt.map_or(0.0, |v| v.remaining_secs());
                        ui_lobby_starting(ui, &mut commands, time_left, &player_query)
                    }
                    RoundState::Ongoing => ui_lobby_ongoing(ui, &mut commands),
                    RoundState::Ended => ui_lobby_end(ui, &mut commands),
                }

                if ui.button("Customize").clicked() {
                    stack.open(MENU_CUSTOMIZATION);
                }
            });
        });

    Ok(())
}

fn ui_lobby_starting(
    ui: &mut Ui,
    commands: &mut Commands,
    time_left: f32,
    player_ready: &Single<Option<&Ready>, With<ThisPlayer>>,
) {
    ui.label(format!("Remaining time: {time_left:.0}"));

    let is_ready = player_ready.is_some();

    let button_label = if is_ready { "Unready" } else { "Ready" };
    if ui.button(button_label).clicked() {
        commands.write_message(ReadyInput(!is_ready));
    }
}

fn ui_lobby_ongoing(ui: &mut Ui, commands: &mut Commands) {
    if ui.button("Join").clicked() {
        commands.write_message(JoinInput(true));
    }

    if ui.button("Observe").clicked() {
        commands.write_message(JoinInput(false));
    }
}

fn ui_lobby_end(ui: &mut Ui, commands: &mut Commands) {
    ui.label("Round is over.");

    if ui.button("Observe").clicked() {
        commands.write_message(JoinInput(false));
    }
}

fn draw_lobby_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    let lobby_art: Handle<Image> = asset_server.load(LOBBY_ART);

    commands
        .spawn((
            LobbyArt,
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: lobby_art,
                    ..default()
                },
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
            ));
        });
}

fn delete_lobby_art(mut commands: Commands, lobby_art: Single<Entity, With<LobbyArt>>) {
    commands.entity(*lobby_art).despawn();
}
