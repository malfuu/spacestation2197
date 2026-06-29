use bevy::ecs::lifecycle::RemovedComponents;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{Checked, Pressed};
use bevy::ui_widgets::RadioButton as BevyRadioButton;

/// Properties accepted by the radio button scene constructor.
pub struct ToolboxRadioProps {
    pub caption: Box<dyn SceneList>,
}

impl Default for ToolboxRadioProps {
    fn default() -> Self {
        Self {
            caption: Box::new(bsn_list!()),
        }
    }
}

pub type RadioProps = ToolboxRadioProps;

/// The RadioButton Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ToolboxRadioProps)]
#[reflect(Component, Default)]
pub struct ToolboxRadioButton;

pub type RadioButton = ToolboxRadioButton;

/// Marker for radio mark indicator.
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct RadioMark;

impl ToolboxRadioButton {
    pub fn scene(props: ToolboxRadioProps) -> impl Scene {
        bsn! {
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                column_gap: px(6.0),
            }
            BevyRadioButton
            ToolboxRadioButton
            Hovered
            Children [(
                Node {
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: px(18.0),
                    height: px(18.0),
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::MAX,
                }
                BorderColor::all(Color::srgb(0.35, 0.35, 0.40))
                Children [(
                    Node {
                        width: px(8.0),
                        height: px(8.0),
                        border_radius: BorderRadius::MAX,
                    }
                    RadioMark
                    BackgroundColor(Color::srgb(0.15, 0.45, 0.85))
                    Visibility::Hidden
                )]),
                {props.caption}
            ]
        }
    }
}

type RadioVisualsQueryData = (Entity, Has<Checked>);
type RadioVisualsFilter = (
    With<ToolboxRadioButton>,
    Or<(
        Added<ToolboxRadioButton>,
        Added<Checked>,
        Changed<Hovered>,
        Added<Pressed>,
    )>,
);

/// Reactive system updating radio dot visibility.
fn update_radio_visuals(
    q_radios: Query<RadioVisualsQueryData, RadioVisualsFilter>,
    q_children: Query<&Children>,
    mut q_marks: Query<&mut Visibility, With<RadioMark>>,
) {
    for (entity, checked) in q_radios.iter() {
        set_radio_mark(entity, checked, &q_children, &mut q_marks);
    }
}

fn update_radio_visuals_remove(
    q_radios: Query<(Entity, Has<Checked>), With<ToolboxRadioButton>>,
    q_children: Query<&Children>,
    mut q_marks: Query<&mut Visibility, With<RadioMark>>,
    mut removed_checked: RemovedComponents<Checked>,
    mut removed_pressed: RemovedComponents<Pressed>,
) {
    removed_checked
        .read()
        .chain(removed_pressed.read())
        .for_each(|ent| {
            if let Ok((entity, checked)) = q_radios.get(ent) {
                set_radio_mark(entity, checked, &q_children, &mut q_marks);
            }
        });
}

fn set_radio_mark(
    entity: Entity,
    checked: bool,
    q_children: &Query<&Children>,
    q_marks: &mut Query<&mut Visibility, With<RadioMark>>,
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

/// Plugin registering systems for radio buttons.
pub struct RadioPlugin;

impl Plugin for RadioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (update_radio_visuals, update_radio_visuals_remove),
        );
    }
}
