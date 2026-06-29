use bevy::camera::Viewport;
use bevy::ecs::lifecycle::Insert;
use bevy::ecs::observer::On;
use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use bevy::ui_widgets::Activate;
use toolbox_ui::controls::button::Button;
use toolbox_ui::prelude::*;

/// Component marker for the main game camera.
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct GameCamera;

#[derive(Component)]
struct RightCamera;

#[derive(Component)]
struct RotatingCube;

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Default)]
struct TargetRightCamera;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(ToolboxUiPlugin)
        .add_systems(Startup, (setup_scene, setup_ui.spawn()))
        .add_systems(Update, (update_camera_viewports, rotate_cube))
        .add_observer(on_target_right_cam);

    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_window: Query<&Window>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };

    let width = window.physical_width();
    let height = window.physical_height();
    let right_width = width * 30 / 100;
    let left_width = width - right_width;

    // Game Camera (3D Viewport covering left 70% of the window)
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            viewport: Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(left_width, height),
                ..Default::default()
            }),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameCamera,
    ));

    // 3D Scene elements visible in the Game Camera viewport
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            ..Default::default()
        },
        Transform::from_xyz(3.0, 8.0, 3.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.5, 1.5, 1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.55, 0.95),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RotatingCube,
    ));

    // Right Camera (2D Viewport covering right 30% of the window)
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            viewport: Some(Viewport {
                physical_position: UVec2::new(left_width, 0),
                physical_size: UVec2::new(right_width, height),
                ..Default::default()
            }),
            ..Default::default()
        },
        RightCamera,
    ));
}

fn setup_ui() -> impl SceneList {
    bsn_list![chatbox()]
}

fn on_target_right_cam(
    insert: On<Insert, TargetRightCamera>,
    mut commands: Commands,
    q_right_cam: Query<Entity, With<RightCamera>>,
) {
    let Ok(cam_ent) = q_right_cam.single() else {
        return;
    };
    commands
        .entity(insert.entity)
        .insert(UiTargetCamera(cam_ent));
}

fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingCube>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(1.0 * time.delta_secs());
        transform.rotate_x(0.5 * time.delta_secs());
    }
}

fn update_camera_viewports(
    q_window: Query<&Window, Changed<Window>>,
    mut q_game: Query<&mut Camera, (With<GameCamera>, Without<RightCamera>)>,
    mut q_right: Query<&mut Camera, (With<RightCamera>, Without<GameCamera>)>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };

    let width = window.physical_width();
    let height = window.physical_height();
    let right_width = width * 30 / 100;
    let left_width = width - right_width;

    if let Ok(mut game_cam) = q_game.single_mut() {
        game_cam.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(left_width, height),
            ..Default::default()
        });
    }

    if let Ok(mut right_cam) = q_right.single_mut() {
        right_cam.viewport = Some(Viewport {
            physical_position: UVec2::new(left_width, 0),
            physical_size: UVec2::new(right_width, height),
            ..Default::default()
        });
    }
}

/// Main Chatbox composite scene spanning the right camera viewport.
fn chatbox() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            right: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            width: percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
        }
        TargetRightCamera
        Children [
            ({chatbox_header()}),
            (
                {pane_body()}
                Node {
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    min_height: px(0.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8.0),
                }
                Children [
                    ({chatbox_history()}),
                    ({chatbox_input()})
                ]
            )
        ]
    }
}

/// Part 1: Chatbox Header bar.
fn chatbox_header() -> impl Scene {
    bsn! {
        {pane_header()}
        Children [
            Text("Chat")
        ]
    }
}

/// Part 2: Chatbox scrollable message history log.
fn chatbox_history() -> impl Scene {
    bsn! {
        Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            min_height: px(0.0),
        }
        @ListView {
            @rows: { Box::new(bsn_list![
                (
                    @ListRow
                    Children [ Text("[OOC] Captain: Welcome to Space Station 2197!") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Engineer: Anyone seen the crowbar?") ]
                ),
                (
                    @ListRow
                    Children [ Text("[OOC] Chef: Pizza party in the kitchen in 5 mins!") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Radio] Officer: Security alert in arrivals corridor.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Scientist: Research lab is powered and operational.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[OOC] Assistant: I just joined the station, hi everyone.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Radio] Medical: Paramedic needed in medbay STAT!") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Engineer: Found the crowbar, fixing primary power generator.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[OOC] Captain: Station status is currently nominal.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Clown: Honk! Honk! Slippy banana peel deployed.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Radio] AI: Life support and atmospheric systems at 100%.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[OOC] Chef: Fresh margherita pizza is ready on table 1!") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Janitor: Cleaning up liquid spill in hydroponics.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Radio] Cargo: Shuttle arriving with industrial crate order.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[OOC] Admin: Remember to follow station roleplay rules.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Botanist: Tomatoes are growing unusually large today.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Radio] Detective: Inspecting evidence in maintenance tunnel B.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Mime: ... (gestures silently towards the airlock)") ]
                ),
                (
                    @ListRow
                    Children [ Text("[OOC] Engineer: Gravity generators are stable.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Captain: All heads of staff report to bridge.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Radio] Quartermaster: Surplus metal sheets available at cargo.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Say] Roboticist: Borg assembly is proceeding on schedule.") ]
                ),
                (
                    @ListRow
                    Children [ Text("[OOC] Assistant: Thanks for the help with navigation!") ]
                ),
                (
                    @ListRow
                    Children [ Text("[Radio] Warden: Brig is clear and secured.") ]
                )
            ]) as Box<dyn SceneList> }
        }
    }
}

/// Part 3: Chatbox bottom input control bar.
fn chatbox_input() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(6.0),
            flex_shrink: 0.0,
        }
        Children [
            (
                @Button {
                    @variant: ButtonVariant::Normal,
                    @caption: bsn! { Text("OOC") }
                }
                on(|ev: On<Activate>, mut q_text: Query<&mut Text>, q_children: Query<&Children>| {
                    for child in q_children.iter_descendants(ev.entity) {
                        let Ok(mut text) = q_text.get_mut(child) else {
                            continue;
                        };
                        if text.0 == "OOC" {
                            text.0 = "Say".to_string();
                            info!("Channel switched to: Say");
                        } else {
                            text.0 = "OOC".to_string();
                            info!("Channel switched to: OOC");
                        }
                    }
                })
            ),
            (
                Node {
                    flex_grow: 1.0,
                }
                @TextInputContainer
                Children [
                    @TextInput
                ]
            ),
            (
                @Button {
                    @variant: ButtonVariant::Primary,
                    @caption: bsn! { Text("Send") }
                }
                on(|_activate: On<Activate>| {
                    info!("Send message clicked!");
                })
            )
        ]
    }
}
