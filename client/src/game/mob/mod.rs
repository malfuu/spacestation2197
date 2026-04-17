use bevy::prelude::*;

use shared::game::mob::{Mob, health::Health};

use crate::{base::chatbox::Chatbox, game::examine::ExamineEntityFurther};

pub(super) struct ClientMobPlugin;

impl Plugin for ClientMobPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_mob_examine_observer);
    }
}

fn add_mob_examine_observer(on: On<Add, Mob>, mut commands: Commands) {
    commands.entity(on.entity).observe(on_mob_examined);
}

fn on_mob_examined(
    on: On<ExamineEntityFurther>,
    health_q: Query<&Health>,
    mut chatbox: ResMut<Chatbox>,
) {
    let entity = on.0;

    if let Ok(health) = health_q.get(entity) {
        let health_value = health.amount;

        let status = if health_value >= 100 {
            "They look healthy."
        } else if health_value >= 50 {
            "They look slightly bruised."
        } else if health_value > 0 {
            "They look heavily bruised."
        } else if health_value > -100 {
            "They are in critical condition!"
        } else {
            "They are dead."
        };

        chatbox.append(status.to_string());
    }
}
