use bevy::ecs::template::EntityTemplate;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{ControlOrientation, Scrollbar as BevyScrollbar, ScrollbarThumb};

/// Props accepted by Scrollbar.
#[derive(Clone, Default)]
pub struct ScrollbarProps {
    pub target: EntityTemplate,
    pub orientation: ControlOrientation,
}

/// The Scrollbar Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ScrollbarProps)]
#[reflect(Component, Default)]
pub struct Scrollbar;

impl Scrollbar {
    pub fn scene(props: ScrollbarProps) -> impl Scene {
        bsn! {
            BevyScrollbar {
                target: {props.target},
                orientation: {props.orientation},
                min_thumb_length: 8.0
            }
            Node {
                border_radius: BorderRadius::all(px(3.0))
            }
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15))
            Children [(
                Hovered
                BackgroundColor(Color::srgb(0.40, 0.40, 0.45))
                ScrollbarThumb {
                    border_radius: BorderRadius::all(px(3.0))
                }
            )]
        }
    }
}

/// Plugin registering systems for scrollbars.
pub struct ScrollbarPlugin;

impl Plugin for ScrollbarPlugin {
    fn build(&self, _app: &mut App) {}
}
