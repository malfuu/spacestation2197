use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::base::session::Disconnect;

const LOADING_ART: &str = "images/backgrounds/loading.png";

pub(super) fn ui_loading(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    egui::Window::new("Loading")
        .collapsible(false)
        .resizable(false)
        .max_width(160.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut()?, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                commands.trigger(Disconnect);
            }

            ui.label("Connecting to server...");
        });

    Ok(())
}

#[derive(Component)]
pub(super) struct LoadingArt;

pub(super) fn draw_loading_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let loading_art: Handle<Image> = asset_server.load(LOADING_ART);

    commands
        .spawn((
            LoadingArt,
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
                    image: loading_art,
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

pub(super) fn delete_loading_background(
    mut commands: Commands,
    loading_art: Single<Entity, With<LoadingArt>>,
) {
    commands.entity(*loading_art).despawn();
}
