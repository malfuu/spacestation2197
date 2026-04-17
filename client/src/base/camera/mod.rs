//! Camera setup and management.
use bevy::{camera::Exposure, input::mouse::MouseWheel, prelude::*};

use shared::defines::CEILING_HEIGHT_HALF;

use crate::base::states::AppState;

pub const CAMERA_TARGET_HEIGHT: f32 = CEILING_HEIGHT_HALF;
pub const CAMERA_DISTANCE_DEFAULT: f32 = 5.0;
pub const CAMERA_DISTANCE_MAX: f32 = 8.0;
pub const CAMERA_DISTANCE_MIN: f32 = 4.0;
pub const ZOOM_SPEED: f32 = 0.4;
pub const ZOOM_SMOOTHING: f32 = 10.0;
pub const CAMERA_OFFSET_DIRECTION: Vec3 = Vec3::new(0.0, 2.0, 1.0);

pub(super) struct ClientCameraPlugin;

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(Startup, setup)
            .add_systems(Update, (handle_camera_zoom, update_camera).chain())
            .add_systems(OnEnter(AppState::Menu), reset_camera);
    }
}

#[derive(Default, Debug)]
pub enum CameraMode {
    #[default]
    Free,
    Follow(Entity),
}

#[derive(Resource, Debug)]
pub struct CameraState {
    pub mode: CameraMode,
    pub position: Vec2,
    pub angle: f32,
    pub distance_real: f32,
    pub distance_target: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            mode: CameraMode::default(),
            position: Vec2::ZERO,
            angle: 0.0,
            distance_real: CAMERA_DISTANCE_DEFAULT,
            distance_target: CAMERA_DISTANCE_DEFAULT,
        }
    }
}

impl CameraState {
    pub fn follow(entity: Entity) -> Self {
        Self {
            mode: CameraMode::Follow(entity),
            ..Default::default()
        }
    }

    pub fn free(position: Vec2) -> Self {
        Self {
            mode: CameraMode::Free,
            position,
            ..Default::default()
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Exposure::INDOOR,
        Camera::default(),
        Camera3d::default(),
        MeshPickingCamera,
    ));
}

fn handle_camera_zoom(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
) {
    let mut zoom_delta = 0.0;
    for event in mouse_wheel_events.read() {
        zoom_delta -= event.y;
    }

    if zoom_delta != 0.0 {
        camera_state.distance_target = (camera_state.distance_target + zoom_delta * ZOOM_SPEED)
            .clamp(CAMERA_DISTANCE_MIN, CAMERA_DISTANCE_MAX);
    }
}

fn update_camera(
    mut camera_transform: Single<&mut Transform, With<Camera>>,
    mobs: Query<&Transform, Without<Camera>>,
    mut camera: ResMut<CameraState>,
    time: Res<Time>,
) {
    camera.distance_real +=
        (camera.distance_target - camera.distance_real) * ZOOM_SMOOTHING * time.delta_secs();

    let pos = match camera.mode {
        CameraMode::Free => Vec3::new(camera.position.x, 0.0, camera.position.y),
        CameraMode::Follow(entity) => match mobs.get(entity) {
            Ok(trans) => trans.translation,
            Err(_) => {
                warn!("following non existent mob? {entity:?}");
                Vec3::ZERO
            }
        },
    };

    let camera_offset = CAMERA_OFFSET_DIRECTION.normalize();
    let rotated = Quat::from_rotation_y(camera.angle) * camera_offset;

    let camera_distance = camera.distance_real;

    let camera_position = Vec3::new(pos.x, 0.0, pos.z) + rotated * camera_distance;
    let look_target = Vec3::new(pos.x, CAMERA_TARGET_HEIGHT, pos.z);

    let mut transform = Transform::from_translation(camera_position);
    transform.look_at(look_target, Vec3::Y);

    **camera_transform = transform;
}

fn reset_camera(mut transform: Single<&mut Transform, With<Camera>>) {
    **transform = Transform::IDENTITY;
}
