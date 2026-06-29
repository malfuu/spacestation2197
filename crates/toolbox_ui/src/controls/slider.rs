use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{
    Slider as BevySlider, SliderOrientation, SliderRange, SliderValue, TrackClick,
};

/// Properties accepted by the slider scene constructor.
pub struct ToolboxSliderProps {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl Default for ToolboxSliderProps {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 1.0,
        }
    }
}

pub type SliderProps = ToolboxSliderProps;

/// The Slider Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ToolboxSliderProps)]
#[require(BevySlider)]
#[reflect(Component, Default)]
pub struct ToolboxSlider;

pub type Slider = ToolboxSlider;

/// Marker for slider value text display.
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct SliderValueText;

impl ToolboxSlider {
    pub fn scene(props: ToolboxSliderProps) -> impl Scene {
        bsn! {
            Node {
                height: px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(8.0)),
                flex_grow: 1.0,
                border_radius: BorderRadius::all(px(6.0)),
            }
            Hovered
            BevySlider {
                track_click: TrackClick::Drag,
                orientation: SliderOrientation::Horizontal,
            }
            ToolboxSlider
            SliderValue({props.value})
            SliderRange::new(props.min, props.max)
            BackgroundColor(Color::srgb(0.20, 0.20, 0.24))
            Children [(
                Node {
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                }
                Text("0.0")
                SliderValueText
            )]
        }
    }
}

fn update_slider_pos(
    mut q_sliders: Query<
        (Entity, &SliderValue),
        (
            With<ToolboxSlider>,
            Or<(Changed<SliderValue>, Added<ToolboxSlider>)>,
        ),
    >,
    q_children: Query<&Children>,
    mut q_slider_text: Query<&mut Text, With<SliderValueText>>,
) {
    for (slider_ent, value) in q_sliders.iter_mut() {
        for child in q_children.iter_descendants(slider_ent) {
            if let Ok(mut text) = q_slider_text.get_mut(child) {
                text.0 = format!("{:.2}", value.0);
            }
        }
    }
}

/// Plugin registering reactive systems for sliders.
pub struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_slider_pos);
    }
}
