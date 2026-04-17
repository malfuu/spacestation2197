//! Window management inside the game client.
//! Including a simple window stack implementation.
use std::collections::HashMap;

use bevy::{ecs::system::SystemId, prelude::*};
use bevy_egui::prelude::*;

use crate::base::{menus::escape_menu::MENU_ESCAPE, states::AppState};

pub(super) struct ClientWindowsPlugin;

impl Plugin for ClientWindowsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowStack>()
            .add_systems(EguiPrimaryContextPass, display_egui_windows)
            .add_systems(OnEnter(AppState::Menu), clear_windows);
    }
}

/// For now windows are uniquely system ids for egui windows
/// that do not have unique access per window.
pub type WindowId = SystemId;

#[derive(Resource, Default)]
pub struct WindowStack {
    windows: Vec<WindowId>,
    ids: HashMap<String, WindowId>,
}

impl WindowStack {
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    fn push(&mut self, window: WindowId) {
        if self.windows.contains(&window) {
            // perhaps push the window to the top?
            warn!("Tried to open already open window {window:?}");
            return;
        }

        self.windows.push(window);
    }

    fn remove(&mut self, window: WindowId) {
        self.windows.retain(|w| *w != window);
    }

    fn get(&self, name: impl Into<String>) -> WindowId {
        *self
            .ids
            .get(&name.into())
            .expect("window needs to be registered!")
    }

    pub fn pop(&mut self, commands: &mut Commands) {
        self.windows.pop();
    }

    pub fn pop_or_escape(&mut self, commands: &mut Commands) {
        if self.windows.pop().is_none() {
            self.open(MENU_ESCAPE);
        }
    }

    pub fn register(&mut self, name: impl Into<String>, id: WindowId) {
        let previous = self.ids.insert(name.into(), id);
        if previous.is_some() {
            warn!("overriding window {previous:?}");
        }
    }

    pub fn open(&mut self, name: impl Into<String>) {
        let id = self.get(name);
        self.push(id);
    }

    pub fn close(&mut self, name: impl Into<String>) {
        let id = self.get(name);
        self.remove(id);
    }

    pub fn toggle(&mut self, name: impl Into<String>) {
        let id = self.get(name);
        if self.windows.contains(&id) {
            self.remove(id);
        } else {
            self.push(id);
        }
    }
}

pub trait WindowCommands {
    fn open_window(&mut self, name: impl Into<String>);
    fn close_window(&mut self, name: impl Into<String>);
    fn toggle_window(&mut self, name: impl Into<String>);
}

impl<'w, 's> WindowCommands for Commands<'w, 's> {
    fn open_window(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.queue(move |world: &mut World| {
            let mut stack = world
                .get_resource_mut::<WindowStack>()
                .expect("stack should exist");
            let id = stack.get(&name);
            stack.push(id);
        });
    }

    fn close_window(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.queue(move |world: &mut World| {
            if let Some(mut stack) = world.get_resource_mut::<WindowStack>() {
                let id = stack.get(&name);
                stack.remove(id);
            }
        });
    }

    fn toggle_window(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.queue(move |world: &mut World| {
            if let Some(mut stack) = world.get_resource_mut::<WindowStack>() {
                stack.toggle(name);
            }
        });
    }
}

fn display_egui_windows(mut commands: Commands, stack: Res<WindowStack>) {
    for window in stack.windows.iter() {
        commands.run_system(*window);
    }
}

fn clear_windows(mut stack: ResMut<WindowStack>) {
    stack.windows.clear();
}
