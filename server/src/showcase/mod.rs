use std::f32::consts::PI;

use bevy::prelude::*;

use content::prelude::*;
use rand::seq::IndexedRandom;
use shared::{
    game::{GameplaySystems, interact::messages::InteractHandMessage, player::Player},
    meta::round::{RoundStartedEvent, is_round_ongoing},
    placeholder::showcase::{SimpleTask, TaskSpawner},
};

const TASK_PROTOTYPE: &str = "task";
const TASK_ROTATE_SPEED: f32 = 0.5;

pub(crate) struct ShowcasePlugin;

impl Plugin for ShowcasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (read_interact_task, rotate_tasks)
                .in_set(GameplaySystems::Logic)
                .run_if(is_round_ongoing),
        )
        .add_observer(spawn_tasks);
    }
}

#[derive(Resource)]
pub struct TaskManager {
    pub tasks_done: u32,
    pub total_tasks: u32,
}

impl TaskManager {
    pub fn new(total_tasks: u32) -> Self {
        Self {
            tasks_done: 0,
            total_tasks,
        }
    }

    pub fn increment(&mut self) {
        self.tasks_done += 1;
    }

    pub fn all_tasks_done(&self) -> bool {
        self.tasks_done >= self.total_tasks
    }
}

fn read_interact_task(
    mut reader: MessageReader<InteractHandMessage>,
    mut commands: Commands,
    tasks: Query<Entity, With<SimpleTask>>,
    manager: Option<ResMut<TaskManager>>,
) {
    let Some(mut manager) = manager else {
        return;
    };

    for msg in reader.read() {
        if !tasks.contains(msg.target) {
            continue;
        }

        manager.increment();
        commands.entity(msg.target).despawn();
    }
}

fn spawn_tasks(
    _: On<RoundStartedEvent>,
    mut commands: Commands,
    spawners: Query<&Transform, With<TaskSpawner>>,
    players: Query<Entity, With<Player>>,
) {
    let mut rng = rand::rng();
    let spawner_list: Vec<&Transform> = spawners.iter().collect();

    if spawner_list.is_empty() {
        warn!("No TaskSpawners found in the map!");
        return;
    }

    let mut total_spawned = 0;
    let tasks_per_player = 5;

    for _player in players.iter() {
        let chosen_spawners = spawner_list.choose_multiple(&mut rng, tasks_per_player);

        for transform in chosen_spawners {
            let mut spawn_pos = **transform;
            spawn_pos.translation += Vec3::new(0.0, 0.5, 0.0);

            commands.spawn_prototype(TASK_PROTOTYPE.to_string(), spawn_pos);
            total_spawned += 1;
        }
    }

    commands.insert_resource(TaskManager::new(total_spawned));
}

fn rotate_tasks(tasks: Query<&mut Transform, With<SimpleTask>>, time: Res<Time<Fixed>>) {
    let rotation = TASK_ROTATE_SPEED * time.delta_secs() * 2. * PI;
    for mut transform in tasks {
        transform.rotate_z(rotation);
    }
}
