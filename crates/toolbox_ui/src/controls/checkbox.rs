use bevy::ecs::lifecycle::RemovedComponents;
use bevy::math::Rot2;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{Checked, Pressed, UiTransform};
use bevy::ui_widgets::Checkbox as BevyCheckbox;

/// A checkbox widget.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ToolboxCheckboxProps)]
#[reflect(Component, Default)]
pub struct ToolboxCheckbox;

/// Type alias for ease of use.
pub type Checkbox = ToolboxCheckbox;

/// Props used to construct a [`ToolboxCheckbox`] scene.
pub struct ToolboxCheckboxProps {
    pub caption: Box<dyn SceneList>,
}

impl Default for ToolboxCheckboxProps {
    fn default() -> Self {
        Self {
            caption: Box::new(bsn_list!()),
        }
    }
}

pub type CheckboxProps = ToolboxCheckboxProps;

/// Marker for the checkbox frame.
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Default)]
struct CheckboxFrame;

/// Marker for the checkbox outline.
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Default)]
struct CheckboxOutline;

/// Marker for the checkbox check mark indicator.
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Default)]
struct CheckboxMark;

impl ToolboxCheckbox {
    pub fn scene(props: ToolboxCheckboxProps) -> impl Scene {
        bsn! {
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                column_gap: px(6.0),
            }
            BevyCheckbox
            CheckboxFrame
            ToolboxCheckbox
            Hovered
            Children [(
                Node {
                    width: px(18.0),
                    height: px(18.0),
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::all(px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                CheckboxOutline
                BackgroundColor(Color::srgb(0.20, 0.20, 0.24))
                BorderColor::all(Color::srgb(0.35, 0.35, 0.40))
                Children [(
                    // L-shaped checkmark: rotated node with bottom & right border
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(4.0),
                        top: px(0.0),
                        width: px(6.0),
                        height: px(11.0),
                        border: UiRect {
                            left: px(0.0),
                            top: px(0.0),
                            right: px(2.0),
                            bottom: px(2.0),
                        },
                    }
                    UiTransform::from_rotation(Rot2::FRAC_PI_4)
                    CheckboxMark
                    BorderColor::all(Color::srgb(0.95, 0.95, 1.0))
                    Visibility::Hidden
                )]),
                {props.caption}
            ]
        }
    }
}

/// Reactive system that updates checkmark visibility and colors based on checked/hover states.
fn update_checkbox_visuals(
    q_checkboxes: Query<
        (Entity, Has<Checked>, &Hovered),
        (
            With<CheckboxFrame>,
            Or<(
                Added<CheckboxFrame>,
                Added<Checked>,
                Changed<Hovered>,
                Added<Pressed>,
            )>,
        ),
    >,
    q_children: Query<&Children>,
    mut q_marks: Query<&mut Visibility, With<CheckboxMark>>,
) {
    for (entity, checked, _hovered) in q_checkboxes.iter() {
        set_checkbox_mark(entity, checked, &q_children, &mut q_marks);
    }
}

fn update_checkbox_visuals_remove(
    q_checkboxes: Query<(Entity, Has<Checked>, &Hovered), With<CheckboxFrame>>,
    q_children: Query<&Children>,
    mut q_marks: Query<&mut Visibility, With<CheckboxMark>>,
    mut removed_checked: RemovedComponents<Checked>,
    mut removed_pressed: RemovedComponents<Pressed>,
) {
    removed_checked
        .read()
        .chain(removed_pressed.read())
        .for_each(|ent| {
            if let Ok((entity, checked, _hovered)) = q_checkboxes.get(ent) {
                set_checkbox_mark(entity, checked, &q_children, &mut q_marks);
            }
        });
}

fn set_checkbox_mark(
    entity: Entity,
    checked: bool,
    q_children: &Query<&Children>,
    q_marks: &mut Query<&mut Visibility, With<CheckboxMark>>,
) {
    for child in q_children.iter_descendants(entity) {
        if let Ok(mut vis) = q_marks.get_mut(child) {
            *vis = if checked {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// Plugin registering reactive systems for checkboxes.
pub struct CheckboxPlugin;

impl Plugin for CheckboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (update_checkbox_visuals, update_checkbox_visuals_remove),
        );
    }
}
