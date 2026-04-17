//! Currently, examining entities is purely clientside.
//! With all output pushed to the chatbox.
use crate::base::chatbox::Chatbox;
use bevy::prelude::*;
use common::EntityTag;

pub(super) struct ClientExaminePlugin;

impl Plugin for ClientExaminePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_examine);
    }
}

#[derive(Event)]
pub struct ExamineEntity(pub Entity);

#[derive(EntityEvent)]
pub struct ExamineEntityFurther(pub Entity);

fn on_examine(
    on: On<ExamineEntity>,
    mut commands: Commands,
    tags: Query<(&EntityTag, Option<&Name>)>,
    mut chatbox: ResMut<Chatbox>,
) {
    let entity_to_examine = on.0;

    if let Ok((tag, name_opt)) = tags.get(entity_to_examine) {
        commands
            .entity(entity_to_examine)
            .trigger(ExamineEntityFurther);

        let entity_tag = (**tag).to_string();

        let message = if let Some(name) = name_opt {
            format!("You examine {} it is a {}.", name.as_str(), entity_tag)
        } else {
            format!("You examine the {}.", entity_tag)
        };

        chatbox.append(message);
    }
}
