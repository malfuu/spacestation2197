use crate::theme::{ThemeBackgroundColor, ThemeBorderColor};
use crate::tokens;
use bevy::prelude::*;

/// A standard pane container for windows or sidebars.
pub fn pane() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
        }
    }
}

/// Pane header component.
pub fn pane_header() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(px(6.0)),
            border: UiRect {
                left: px(1.0),
                top: px(1.0),
                right: px(1.0),
                bottom: px(0.0),
            },
            min_height: px(30.0),
            column_gap: px(6.0),
            border_radius: BorderRadius::px(4.0, 4.0, 0.0, 0.0),
        }
        ThemeBackgroundColor(tokens::PANE_HEADER_BG)
        ThemeBorderColor(tokens::PANE_HEADER_BORDER)
    }
}

/// Vertical divider between groups of widgets in pane headers.
pub fn pane_header_divider() -> impl Scene {
    bsn! {
        Node {
            width: px(1.0),
            align_self: AlignSelf::Stretch,
        }
        Children [(
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                right: px(0.0),
                top: px(-6.0),
                bottom: px(-6.0),
            }
            ThemeBackgroundColor(tokens::PANE_HEADER_DIVIDER)
        )]
    }
}

/// Pane body container.
pub fn pane_body() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            padding: UiRect::all(px(6.0)),
            border_radius: BorderRadius::px(0.0, 0.0, 4.0, 4.0),
        }
        ThemeBackgroundColor(tokens::PANE_BODY_BG)
    }
}
