use crate::controls::scrollbar::Scrollbar;
use crate::palette;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{ControlOrientation, ListBox, ListItem, ScrollArea};

/// Props accepted by ListView.
pub struct ListViewProps {
    pub rows: Box<dyn SceneList>,
}

impl Default for ListViewProps {
    fn default() -> Self {
        Self {
            rows: Box::new(bsn_list!()),
        }
    }
}

/// The ListView Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ListViewProps)]
#[reflect(Component, Default)]
pub struct ListView;

impl ListView {
    pub fn scene(props: ListViewProps) -> impl Scene {
        bsn! {
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Start,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_height: px(0.0),
                padding: UiRect {
                    right: px(10.0)
                }
            }
            ListBox
            Children [
                (
                    #inner
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::Start,
                        flex_grow: 1.0,
                        flex_shrink: 1.0,
                        min_height: px(0.0),
                        overflow: Overflow::scroll_y(),
                    }
                    ScrollArea
                    Children [
                        {props.rows}
                    ]
                ),
                @Scrollbar {
                    @target: #inner,
                    @orientation: {ControlOrientation::Vertical}
                }
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0.0),
                    top: px(0.0),
                    bottom: px(0.0),
                    width: px(6.0),
                }
            ]
        }
    }
}

/// The ListRow Scene Component.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ListRowProps)]
#[reflect(Component, Default)]
pub struct ListRow;

#[derive(Default)]
pub struct ListRowProps {
    pub bg: Option<Color>,
}


impl ListRow {
    pub fn scene(props: ListRowProps) -> impl Scene {
        let initial_bg = props.bg.unwrap_or(palette::DEEP_SLATE_1);
        bsn! {
            Node {
                min_height: px(24.0),
                width: percent(100.0),
                flex_shrink: 0.0,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                padding: UiRect::axes(px(8.0), px(2.0)),
            }
            TextColor(palette::TEXT_PRIMARY)
            BackgroundColor(initial_bg)
            Hovered
            ListItem
        }
    }
}

/// Plugin registering systems for list views.
pub struct ListViewPlugin;

impl Plugin for ListViewPlugin {
    fn build(&self, _app: &mut App) {}
}
