//! Player input processing
use bevy::{input::InputSystems, prelude::*, window::WindowMode};

use common::EntityTag;
use shared::{
    defines::{CEILING_HEIGHT, CEILING_HEIGHT_HALF},
    game::mob::MoveInput,
};

use crate::{
    base::{
        camera::{CameraState, GameCamera},
        windows::WindowStack,
    },
    game::examine::ExamineEntity,
};

pub(super) struct ClientInputPlugin;

impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtraInputs>()
            .add_plugins(MeshPickingPlugin)
            .add_systems(
                PreUpdate,
                (clientside_inputs, update_mouse_inputs).after(InputSystems),
            )
            .add_systems(Update, (update_inputs, input_keyboard, toggle_fullscreen))
            .add_observer(on_mouse_hover)
            .add_observer(on_mouse_exit)
            .add_observer(on_mouse_click);
    }
}

/// triggered when clicked on valid entity.
#[derive(Event, Debug)]
pub struct EntityClick {
    pub entity: Entity,
}

pub struct MousePositions {
    /// global tile position that the mouse hovers.
    pub tile_position: IVec2,
    /// mouse position in the XZ plane.
    pub ground_plane_position: Vec3,
    /// mouse position in [`CEILING_HEIGHT_HALF`] height plane.
    pub midair_plane_position: Vec3,
    /// mouse position in [`CEILING_HEIGHT`] height plane.
    pub ceiling_plane_position: Vec3,
}

#[derive(Resource, Default)]
pub struct ExtraInputs {
    mouse_positions: Option<MousePositions>,
    hovering: Option<Entity>,
}

impl ExtraInputs {
    pub fn mouse_positions(&self) -> &Option<MousePositions> {
        &self.mouse_positions
    }

    pub fn hovering(&self) -> Option<Entity> {
        self.hovering
    }
}

fn on_mouse_hover(
    event: On<Pointer<Over>>,
    mut commands: Commands,
    entities: Query<Entity, With<EntityTag>>,
    mut inputs: ResMut<ExtraInputs>,
) {
    if !entities.contains(event.entity) {
        return;
    }

    inputs.hovering = Some(event.entity);
}

fn on_mouse_exit(
    event: On<Pointer<Out>>,
    mut commands: Commands,
    entities: Query<Entity, With<EntityTag>>,
    mut inputs: ResMut<ExtraInputs>,
) {
    if !entities.contains(event.entity) {
        return;
    }

    inputs.hovering = None;
}

fn on_mouse_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    entities: Query<Entity, With<EntityTag>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !entities.contains(event.entity) {
        return;
    }

    if keys.pressed(KeyCode::ShiftLeft) {
        commands.trigger(ExamineEntity(event.entity));
    } else {
        commands.trigger(EntityClick {
            entity: event.entity,
        });
    }
}

fn clientside_inputs(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut camera: ResMut<CameraState>,
    time: Res<Time>,
) {
    let mut dir = 0.0;
    if input.pressed(KeyCode::Digit1) {
        dir -= 1.0;
    }

    if input.pressed(KeyCode::Digit3) {
        dir += 1.0;
    }

    camera.angle += dir * time.delta_secs();
}

fn update_inputs(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut stack: ResMut<WindowStack>,
) {
    if input.just_pressed(KeyCode::Escape) {
        stack.pop_or_escape(&mut commands);
    }
}

// TODO: why is mob movement input here
fn input_keyboard(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    camera: Res<CameraState>,
) {
    let mut direction = Vec2::ZERO;

    for (key, dir) in [
        (KeyCode::KeyW, Vec2::NEG_Y),
        (KeyCode::KeyS, Vec2::Y),
        (KeyCode::KeyA, Vec2::NEG_X),
        (KeyCode::KeyD, Vec2::X),
    ] {
        if keys.pressed(key) {
            direction += dir;
        }
    }

    // Only send message if there is movement
    if let Some(normalized) = direction.try_normalize() {
        let rotated = Vec2::from_angle(-camera.angle).rotate(normalized);
        commands.write_message(MoveInput(rotated));
    }
}

fn toggle_fullscreen(keys: Res<ButtonInput<KeyCode>>, mut window: Single<&mut Window>) {
    if keys.just_pressed(KeyCode::F11) {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            _ => WindowMode::Windowed,
        };
    }
}

fn update_mouse_inputs(
    window: Single<&Window>,
    camera_q: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut inputs: ResMut<ExtraInputs>,
) {
    inputs.mouse_positions = None;

    let (camera, camera_transform) = *camera_q;

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let ray = camera
        .viewport_to_world(camera_transform, cursor_position)
        .expect("should give ray");

    let plane = InfinitePlane3d::new(Vec3::Y);

    let get_intersection = |y_height: f32| -> Option<Vec3> {
        let plane_origin = vec3(0.0, y_height, 0.0);
        let distance = ray.intersect_plane(plane_origin, plane)?;
        Some(ray.get_point(distance))
    };

    let Some(ground_pos) = get_intersection(0.0) else {
        return;
    };

    let Some(midair_pos) = get_intersection(CEILING_HEIGHT_HALF) else {
        return;
    };

    let Some(ceiling_pos) = get_intersection(CEILING_HEIGHT) else {
        return;
    };

    let global_cursor_int = ground_pos.xz().floor().as_ivec2();

    inputs.mouse_positions = Some(MousePositions {
        tile_position: global_cursor_int,
        ground_plane_position: ground_pos,
        midair_plane_position: midair_pos,
        ceiling_plane_position: ceiling_pos,
    });
}
