use bevy::prelude::*;

/// Sub-pane container.
pub fn subpane() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
        }
    }
}

/// Sub-pane header component.
pub fn subpane_header() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect {
                left: px(1.0),
                top: px(1.0),
                right: px(1.0),
                bottom: px(0.0),
            },
            padding: UiRect::horizontal(px(10.0)),
            min_height: px(30.0),
            column_gap: px(4.0),
            border_radius: BorderRadius::px(4.0, 4.0, 0.0, 0.0),
        }
        BackgroundColor(Color::srgb(0.18, 0.18, 0.21))
        BorderColor::all(Color::srgb(0.28, 0.28, 0.32))
    }
}

/// Sub-pane body container.
pub fn subpane_body() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            border: UiRect {
                left: px(1.0),
                right: px(1.0),
                bottom: px(1.0),
                top: px(0.0),
            },
            row_gap: px(4.0),
            padding: UiRect::all(px(6.0)),
            border_radius: BorderRadius::px(0.0, 0.0, 4.0, 4.0),
        }
        BackgroundColor(Color::srgb(0.13, 0.13, 0.16))
        BorderColor::all(Color::srgb(0.28, 0.28, 0.32))
    }
}
