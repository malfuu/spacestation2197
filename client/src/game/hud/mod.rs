use bevy::prelude::*;

use shared::game::mob::health::Health;

use crate::game::mind::{Controlling, MindState};

pub const HEALTH_HEALTHY_COLOR: Color = Color::srgb(0.1, 0.9, 0.1);
pub const HEALTH_SLIGHTLY_BRUISED_COLOR: Color = Color::srgb(0.6, 0.8, 0.2);
pub const HEALTH_HEAVILY_BRUISED_COLOR: Color = Color::srgb(0.8, 0.4, 0.1);
pub const HEALTH_CRITICAL_COLOR: Color = Color::srgb(0.6, 0.0, 0.0);
pub const HEALTH_DEAD_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

pub(super) struct ClientHudPlugin;

impl Plugin for ClientHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MindState::Controlling), create_health_ui)
            .add_systems(
                Update,
                update_health_ui.run_if(in_state(MindState::Controlling)),
            )
            .add_systems(OnExit(MindState::Controlling), delete_health_ui);
    }
}

#[derive(Component)]
struct HealthRootNode;

fn health_ui_data(health: &Health) -> (&str, Color) {
    let val = health.amount;
    if val >= 100 {
        ("Healthy", HEALTH_HEALTHY_COLOR)
    } else if val >= 50 {
        ("Slightly bruised", HEALTH_SLIGHTLY_BRUISED_COLOR)
    } else if val > 0 {
        ("Heavily bruised", HEALTH_HEAVILY_BRUISED_COLOR)
    } else if val > -100 {
        ("Critical", HEALTH_CRITICAL_COLOR)
    } else {
        ("Dead", HEALTH_DEAD_COLOR)
    }
}

fn create_health_ui(mut commands: Commands, health_q: Single<Option<&Health>, With<Controlling>>) {
    let Some(health) = *health_q else {
        return;
    };

    let (label, color) = health_ui_data(health);

    commands.spawn((
        HealthRootNode,
        Node {
            position_type: PositionType::Absolute,
            right: vw(1.0),
            top: vh(50.0),
            ..default()
        },
        Text(label.to_string()),
        TextColor(color),
        BackgroundColor(Srgba::gray(0.1).into()),
    ));
}

fn update_health_ui(
    health_q: Single<Option<Ref<Health>>, With<Controlling>>,
    ui_node: Option<Single<(&mut Text, &mut TextColor), With<HealthRootNode>>>,
) {
    let (Some(mut ui), Some(health_ref)) = (ui_node, health_q.as_ref()) else {
        return;
    };

    if !health_ref.is_changed() {
        return;
    }

    let (label, color) = health_ui_data(health_ref);

    ui.0.0 = label.to_string();
    ui.1.0 = color;
}

fn delete_health_ui(mut commands: Commands, ui_node: Option<Single<Entity, With<HealthRootNode>>>) {
    if let Some(node) = ui_node {
        commands.entity(*node).despawn();
    }
}
