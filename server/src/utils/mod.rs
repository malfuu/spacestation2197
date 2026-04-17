use bevy::prelude::*;
use bevy_replicon::prelude::*;

use common::*;
use content::prelude::*;
use shared::{
    chat::ChatMessage, game::mob::color::SkinColor, meta::customization::PlayerSettings,
    utils::filters::*,
};

use crate::game::mind::Controlled;

pub enum SpawnMethod {
    Position(Vec2),
    Spawner(PrototypeId),
}

pub trait SpawnerCommandsExt {
    fn spawn_player(&mut self, player: Entity, prototype: PrototypeId, method: SpawnMethod);

    fn apply_customization(&mut self, player: Entity, mob: Entity);
}

impl SpawnerCommandsExt for Commands<'_, '_> {
    fn spawn_player(&mut self, player: Entity, prototype: PrototypeId, method: SpawnMethod) {
        self.queue(move |world: &mut World| {
            let position = match method {
                SpawnMethod::Position(pos) => pos,
                SpawnMethod::Spawner(spawner_id) => {
                    let mut query =
                        world.query_filtered::<(&EntityTag, &Transform), MarkerFilter>();

                    let found_pos = query
                        .iter(world)
                        .find(|(tag, _)| tag.0 == spawner_id)
                        .map(|(_, transform)| transform.translation.xz());

                    match found_pos {
                        Some(pos) => pos,
                        None => {
                            error!("No spawner found with ID: {:?}", spawner_id);
                            return;
                        }
                    }
                }
            };

            let mut commands = world.commands();
            let mut mob_commands = commands
                .spawn_prototype(prototype, Transform::from_xyz(position.x, 0., position.y));

            let mob = mob_commands.insert(Controlled(player));
            let mob_id = mob.id();

            // this command could've been inlined
            commands.apply_customization(player, mob_id);
        });
    }

    fn apply_customization(&mut self, player: Entity, mob: Entity) {
        self.queue(move |world: &mut World| {
            let settings = world
                .query_filtered::<&PlayerSettings, PlayerFilter>()
                .get(world, player)
                .expect("Player should have settings.");
            let settings = settings.clone();

            let mut commands = world.commands();
            let mut mob_commands = commands.entity(mob);

            // color
            let color = settings.character.skin_color;

            // name
            let name = settings.character.name;

            mob_commands.insert((SkinColor(color), Name::new(name.clone())));
        });
    }
}

pub trait MessageCommandsExt {
    /// Sends a simple [`ChatMessage`] to a specific client
    fn send_chat_message(&mut self, player: Entity, message: impl Into<String>);
    /// Sends a simple [`ChatMessage`] to all clients
    fn broadcast_chat_message(&mut self, message: impl Into<String>);
}

impl MessageCommandsExt for Commands<'_, '_> {
    fn send_chat_message(&mut self, player: Entity, message: impl Into<String>) {
        self.write_message(ToClients {
            mode: SendMode::Direct(ClientId::Client(player)),
            message: ChatMessage(message.into()),
        });
    }

    fn broadcast_chat_message(&mut self, message: impl Into<String>) {
        self.write_message(ToClients {
            mode: SendMode::Broadcast,
            message: ChatMessage(message.into()),
        });
    }
}
