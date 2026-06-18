//! Various debug tools
mod atmos;
mod frametime;
mod gizmos;
mod grid;
mod networking;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::base::windows::{WindowCommands, WindowStack};
use crate::debug_tools::networking::DebugNetworkingPlugin;
use crate::debug_tools::{atmos::DebugAtmosPlugin, gizmos::DebugGizmos};
use crate::debug_tools::{frametime::DebugFrametimePlugin, grid::DebugGridPlugin};

pub const MENU_DEBUG_OPTIONS: &str = "DebugOptions";

pub(super) struct ClientDebugPlugin;

impl Plugin for ClientDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<DebugGizmos>()
            .init_resource::<DebugResource>()
            .add_plugins(DebugAtmosPlugin)
            .add_plugins(DebugGridPlugin)
            .add_plugins(DebugFrametimePlugin)
            .add_plugins(DebugNetworkingPlugin)
            // .add_plugins(WorldInspectorPlugin::new())
            .add_systems(Startup, register_debug_window)
            .add_systems(Update, toggle_debug_window);
    }
}

pub trait AppDebugOptionExt {
    fn register_debug_option(&mut self, option: impl Into<String>) -> &mut Self;
}

impl AppDebugOptionExt for App {
    fn register_debug_option(&mut self, option: impl Into<String>) -> &mut Self {
        self.init_resource::<DebugResource>();
        let mut debug_resource = self.world_mut().resource_mut::<DebugResource>();
        debug_resource.options.insert(option.into(), false);
        self
    }
}

#[derive(Resource, Default)]
pub struct DebugResource {
    pub options: HashMap<String, bool>,
}

pub fn option_enabled(option: impl Into<String>) -> impl Fn(Res<DebugResource>) -> bool + Clone {
    let str = option.into();
    move |resource: Res<DebugResource>| *resource.options.get(&str).unwrap_or(&false)
}

fn register_debug_window(world: &mut World) {
    let id = world.register_system(ui_debug_options);
    let mut stack = world.resource_mut::<WindowStack>();
    stack.register(MENU_DEBUG_OPTIONS, id);
}

fn toggle_debug_window(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F3) {
        commands.toggle_window(MENU_DEBUG_OPTIONS);
    }
}

pub fn ui_debug_options(
    mut contexts: EguiContexts,
    mut debug_res: ResMut<DebugResource>,
) -> Result {
    egui::Window::new("Debug Options").show(contexts.ctx_mut()?, |ui| {
        let mut keys: Vec<String> = debug_res.options.keys().cloned().collect();
        keys.sort();

        for key in keys {
            if let Some(is_enabled) = debug_res.options.get_mut(&key) {
                ui.checkbox(is_enabled, &key);
            }
        }

        if debug_res.options.is_empty() {
            ui.label(egui::RichText::new("No debug options registered.").italics());
        }
    });

    Ok(())
}
