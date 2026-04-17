use bevy::prelude::*;
use bevy_replicon::prelude::*;

use shared::{
    defines::SAY_REACH,
    game::{
        GameplaySystems,
        ghost::Ghost,
        say::{EntityListen, EntitySay, Listener, SayInput, Speaker, deadsay::EntityDeadSay},
    },
    utils::filters::{MobFilter, PlayerFilter},
};

use crate::{
    game::mind::{Controlled, Controls},
    utils::MessageCommandsExt,
};

pub(super) struct SayPlugin;

impl Plugin for SayPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_entity_say)
            .add_observer(on_entity_deadsay)
            .add_observer(on_entity_listen)
            .add_systems(
                FixedUpdate,
                (read_input_say_input,).in_set(GameplaySystems::Inputs),
            );
    }
}

fn read_input_say_input(
    mut reader: MessageReader<FromClient<SayInput>>,
    mut commands: Commands,
    players: Query<&Controls, PlayerFilter>,
) {
    for input in reader.read() {
        let ClientId::Client(client_entity) = input.client_id else {
            continue;
        };

        let Ok(owner) = players.get(client_entity) else {
            continue;
        };

        let Some(mob_entity) = owner.iter().next() else {
            warn!("Received input for no associated mob from {client_entity:?}");
            continue;
        };

        commands.trigger(EntitySay {
            speaker: mob_entity,
            message: input.message.0.clone(),
        });
    }
}

fn on_entity_say(
    speech: On<EntitySay>,
    mut commands: Commands,
    speakers: Query<(Option<&Ghost>, &GlobalTransform), With<Speaker>>,
    listeners: Query<(Entity, &GlobalTransform), With<Listener>>,
) {
    let Ok((spectator_opt, speaker_transform)) = speakers.get(speech.speaker) else {
        info!(
            "Received EntitySay from non-Speaker Entity {:?}",
            speech.speaker
        );
        return;
    };

    if spectator_opt.is_some() {
        commands.trigger(EntityDeadSay {
            speaker: speech.speaker,
            message: speech.message.clone(),
        });
        return;
    }

    for (listener, listener_transform) in listeners {
        let diff = speaker_transform.translation() - listener_transform.translation();
        if diff.length() > SAY_REACH {
            continue;
        }

        commands.trigger(EntityListen {
            speaker: speech.speaker,
            listener,
            message: speech.message.clone(),
        });
    }
}

fn on_entity_listen(
    speech: On<EntityListen>,
    mut commands: Commands,
    mobs: Query<&Controlled, MobFilter>,
    entities: Query<&Name>,
) {
    let Ok(listener_player) = mobs.get(speech.listener) else {
        return;
    };

    let name = match entities.get(speech.speaker) {
        Ok(name) => name.as_str(),
        Err(_) => {
            warn!("Nameless entity {:?} is speaking.", speech.speaker);
            "nameless"
        }
    };

    let message = format!("{} says, \"{}\"", name, speech.message);
    commands.send_chat_message(listener_player.0, message);
}

fn on_entity_deadsay(
    speech: On<EntityDeadSay>,
    mut commands: Commands,
    speaker: Query<&Name, MobFilter>,
    mobs: Query<&Controlled, (MobFilter, With<Ghost>)>,
) {
    let name = speaker
        .get(speech.speaker)
        .expect("Speaker should have a name.");
    let message = format!("[DEAD] {} says, \"{}\"", name.as_str(), speech.message);

    for controlled in mobs.iter() {
        commands.send_chat_message(controlled.0, message.clone());
    }
}
