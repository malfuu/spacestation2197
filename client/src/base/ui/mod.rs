use bevy::prelude::*;

pub(super) struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MapToScreen>()
            .register_type::<MapToScreenAnchor>()
            .add_systems(Startup, spawn_test_speech_bubble)
            .add_systems(PostUpdate, update_map_to_screen);
    }
}

fn spawn_test_speech_bubble(mut commands: Commands) {
    commands.spawn_scene(speech_bubble(bsn_list![
        screen_text("Hello World!".to_string(), Color::WHITE, Color::BLACK),
        screen_text(
            "straight foobaring rn".to_string(),
            Color::WHITE,
            Color::BLACK
        ),
    ]));
}

pub fn speech_bubble(messages: impl SceneList) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
        }
        MapToScreen {
            anchor: MapToScreenAnchor::BottomCenter,
        }
        Children [
            {messages}
        ]
    }
}

fn screen_text(text: String, text_color: Color, bg_color: Color) -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::all(px(4.0)),
            margin: UiRect::bottom(px(2.0)),
        }
        Text(text)
        TextColor(text_color)
        BackgroundColor(bg_color)
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq)]
pub enum MapToScreenAnchor {
    #[default]
    Center,
    BottomCenter,
}

#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(Transform)]
pub struct MapToScreen {
    pub anchor: MapToScreenAnchor,
}

fn update_map_to_screen(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut ui_query: Query<(&GlobalTransform, &mut Node, &ComputedNode, &MapToScreen)>,
) {
    let (camera, camera_transform) = camera_query.single().expect("One camera in the world");

    for (global_transform, mut node, computed_node, map_to_screen) in ui_query.iter_mut() {
        let world_pos = global_transform.translation();
        let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
            continue;
        };
        let size = computed_node.size();
        let (offset_x, offset_y) = match map_to_screen.anchor {
            MapToScreenAnchor::Center => (size.x / 2.0, size.y / 2.0),
            MapToScreenAnchor::BottomCenter => (size.x / 2.0, size.y),
        };
        node.left = px(viewport_pos.x - offset_x);
        node.top = px(viewport_pos.y - offset_y);
    }
}
