//! Our beloved main text recepticle.

use bevy::prelude::*;
use bevy_egui::prelude::*;
use shared::chat::ChatMessage;

use crate::{
    base::session::SessionState,
    placeholder::chat::{OocEnter, SayEnter},
};

pub(super) struct ClientChatboxPlugin;

impl Plugin for ClientChatboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chatbox>()
            .add_systems(EguiPrimaryContextPass, ui_chatbox.run_if(is_chatbox_open))
            .add_systems(Update, on_chatbox_message);
    }
}

#[derive(Resource, Default, Debug)]
pub struct Chatbox {
    lines: Vec<String>,
}

impl Chatbox {
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn append(&mut self, line: String) {
        self.lines.push(line);
    }
}

fn on_chatbox_message(mut reader: MessageReader<ChatMessage>, mut chatboes: ResMut<Chatbox>) {
    for msg in reader.read() {
        chatboes.append(msg.0.clone());
    }
}

fn is_chatbox_open(state: Option<Res<State<SessionState>>>) -> bool {
    matches!(
        state,
        Some(res) if res.get() == &SessionState::Playing
    )
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum ChatChannel {
    Say,
    #[default]
    Ooc,
}

fn ui_chatbox(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut input_text: Local<String>,
    mut selected_channel: Local<ChatChannel>,
    chatbox: Res<Chatbox>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    let context = contexts.ctx_mut()?;

    let mut request_focus = false;

    if !context.egui_wants_keyboard_input() {
        if keys.just_pressed(KeyCode::KeyT) {
            *selected_channel = ChatChannel::Say;
            request_focus = true;
        }
        if keys.just_pressed(KeyCode::KeyO) {
            *selected_channel = ChatChannel::Ooc;
            request_focus = true;
        }
    }

    if request_focus {
        input_text.clear();
    }

    egui::Window::new("Chat")
        .default_width(300.0)
        .default_height(200.0)
        .max_width(300.0)
        .max_height(200.0)
        .show(context, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(200.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.set_min_height(200.0);
                    for line in &chatbox.lines {
                        ui.label(line.to_string());
                    }
                });

            ui.separator();

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("channel_select")
                    .selected_text(match *selected_channel {
                        ChatChannel::Say => "Say",
                        ChatChannel::Ooc => "OOC",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut *selected_channel, ChatChannel::Say, "Say");
                        ui.selectable_value(&mut *selected_channel, ChatChannel::Ooc, "OOC");
                    });

                let response = ui.add(egui::TextEdit::singleline(&mut *input_text));
                if request_focus {
                    response.request_focus();
                }

                if (ui.button("Send").clicked()
                    || response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    && !input_text.is_empty()
                {
                    match *selected_channel {
                        ChatChannel::Say => {
                            commands.trigger(SayEnter(input_text.clone()));
                        }
                        ChatChannel::Ooc => {
                            commands.trigger(OocEnter(input_text.clone()));
                        }
                    }

                    input_text.clear();
                }
            });
        });

    Ok(())
}
