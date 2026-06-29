use bevy::ecs::lifecycle::RemovedComponents;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{Checked, Pressed};
use bevy::ui_widgets::Checkbox as BevyCheckbox;

/// The ToggleSwitch Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct ToolboxToggleSwitch;

pub type ToggleSwitch = ToolboxToggleSwitch;

/// Marker for the toggle switch slide handle.
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct ToggleSwitchSlide;

impl ToolboxToggleSwitch {
    pub fn scene() -> impl Scene {
        bsn! {
            Node {
                width: px(32.0),
                height: px(18.0),
                border: UiRect::all(px(2.0)),
                border_radius: BorderRadius::all(px(9.0)),
            }
            BevyCheckbox
            ToolboxToggleSwitch
            Hovered
            BackgroundColor(Color::srgb(0.20, 0.20, 0.24))
            BorderColor::all(Color::srgb(0.35, 0.35, 0.40))
            Children [(
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    bottom: px(0.0),
                    width: px(14.0),
                    border_radius: BorderRadius::all(px(7.0)),
                }
                ToggleSwitchSlide
                BackgroundColor(Color::srgb(0.60, 0.60, 0.65))
            )]
        }
    }
}

type SwitchVisualsQueryData = (Entity, Has<Checked>);
type SwitchVisualsFilter = (
    With<ToolboxToggleSwitch>,
    Or<(
        Added<ToolboxToggleSwitch>,
        Added<Checked>,
        Changed<Hovered>,
        Added<Pressed>,
    )>,
);

/// Reactive system updating switch position and colors.
fn update_switch_visuals(
    q_switches: Query<SwitchVisualsQueryData, SwitchVisualsFilter>,
    q_children: Query<&Children>,
    mut q_slides: Query<(&mut Node, &mut BackgroundColor), With<ToggleSwitchSlide>>,
) {
    for (entity, checked) in q_switches.iter() {
        set_switch_slide(entity, checked, &q_children, &mut q_slides);
    }
}

fn update_switch_visuals_remove(
    q_switches: Query<(Entity, Has<Checked>), With<ToolboxToggleSwitch>>,
    q_children: Query<&Children>,
    mut q_slides: Query<(&mut Node, &mut BackgroundColor), With<ToggleSwitchSlide>>,
    mut removed_checked: RemovedComponents<Checked>,
    mut removed_pressed: RemovedComponents<Pressed>,
) {
    removed_checked
        .read()
        .chain(removed_pressed.read())
        .for_each(|ent| {
            if let Ok((entity, checked)) = q_switches.get(ent) {
                set_switch_slide(entity, checked, &q_children, &mut q_slides);
            }
        });
}

fn set_switch_slide(
    entity: Entity,
    checked: bool,
    q_children: &Query<&Children>,
    q_slides: &mut Query<(&mut Node, &mut BackgroundColor), With<ToggleSwitchSlide>>,
) {
    for child in q_children.iter_descendants(entity) {
        if let Ok((mut node, mut bg)) = q_slides.get_mut(child) {
            if checked {
                node.left = px(14.0);
                bg.0 = Color::srgb(0.15, 0.45, 0.85);
            } else {
                node.left = px(0.0);
                bg.0 = Color::srgb(0.60, 0.60, 0.65);
            }
        }
    }
}

/// Plugin registering systems for toggle switches.
pub struct ToggleSwitchPlugin;

impl Plugin for ToggleSwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (update_switch_visuals, update_switch_visuals_remove),
        );
    }
}
