use bevy::prelude::*;

pub struct FollowingTextPlugin;

impl Plugin for FollowingTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_following_text);
    }
}

#[derive(Component)]
pub struct FollowingText {
    pub target: Entity,
    pub offset: Vec3,
    pub timer: Timer,
}

pub trait FollowingTextCommandsExt {
    fn spawn_following_text(
        &mut self,
        text: String,
        text_color: Color,
        bg_color: Color,
        target: Entity,
        offset: Vec3,
        duration_secs: f32,
    );
}

impl<'w, 's> FollowingTextCommandsExt for Commands<'w, 's> {
    fn spawn_following_text(
        &mut self,
        text: String,
        text_color: Color,
        bg_color: Color,
        target: Entity,
        offset: Vec3,
        duration_secs: f32,
    ) {
        self.spawn((
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            Text::new(text),
            TextColor(text_color),
            BackgroundColor(bg_color),
            FollowingText {
                target,
                offset,
                timer: Timer::from_seconds(duration_secs, TimerMode::Once),
            },
        ));
    }
}

fn update_following_text(
    time: Res<Time>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    transform_query: Query<&GlobalTransform>,
    mut bubble_query: Query<(Entity, &mut FollowingText, &mut Node, &ComputedNode)>,
    mut commands: Commands,
) {
    let (camera, camera_transform) = camera_query.single().expect("One camera in the world");

    for (bubble_entity, mut following, mut node, computed_node) in bubble_query.iter_mut() {
        following.timer.tick(time.delta());
        if following.timer.is_finished() {
            commands.entity(bubble_entity).despawn();
            continue;
        }

        let world_pos = if let Ok(target_transform) = transform_query.get(following.target) {
            target_transform.translation() + following.offset
        } else {
            Vec3::ZERO + following.offset
        };

        if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) {
            let size = computed_node.size();
            node.left = px(viewport_pos.x - (size.x / 2.0));
            node.top = px(viewport_pos.y - (size.y / 2.0));
        }
    }
}
