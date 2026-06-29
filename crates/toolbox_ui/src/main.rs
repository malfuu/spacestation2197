use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::{Activate, SliderValue, ValueChange};
use toolbox_ui::controls::button::Button;
use toolbox_ui::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(ToolboxUiPlugin)
        .add_systems(Startup, setup_scene.spawn());

    app.run();
}

fn setup_scene() -> impl SceneList {
    bsn_list![Camera2d, ui_root()]
}

fn ui_root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        Children [(
            Node {
                width: px(340.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
            }
            Children [
                (
                    {pane_header()}
                    Children [
                        Text("Toolbox UI Showcase")
                    ]
                ),
                (
                    {pane_body()}
                    Children [
                        (
                            @Button {
                                @variant: ButtonVariant::Primary,
                                @caption: bsn! { Text("Primary Action") }
                            }
                            on(|_activate: On<Activate>| {
                                info!("Primary button clicked!");
                            })
                        ),
                        (
                            @Button {
                                @variant: ButtonVariant::Normal,
                                @caption: bsn! { Text("Normal Action") }
                            }
                        ),
                        (
                            @Checkbox {
                                @caption: bsn! { Text("Enable Features") }
                            }
                            on(|change: On<ValueChange<bool>>, mut commands: Commands| {
                                info!("Checkbox toggled to: {}", change.value);
                                if change.value {
                                    commands.entity(change.source).insert(Checked);
                                } else {
                                    commands.entity(change.source).remove::<Checked>();
                                }
                            })
                        ),
                        (
                            @ToggleSwitch
                            on(|change: On<ValueChange<bool>>, mut commands: Commands| {
                                info!("ToggleSwitch state: {}", change.value);
                                if change.value {
                                    commands.entity(change.source).insert(Checked);
                                } else {
                                    commands.entity(change.source).remove::<Checked>();
                                }
                            })
                        ),
                        (
                            @Slider {
                                @value: 0.5,
                                @min: 0.0,
                                @max: 1.0,
                            }
                            on(|change: On<ValueChange<f32>>, mut commands: Commands| {
                                info!("Slider value changed to: {:.2}", change.value);
                                commands.entity(change.source).insert(SliderValue(change.value));
                            })
                        )
                    ]
                )
            ]
        )]
    }
}
