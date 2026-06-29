use bevy::prelude::*;

/// Group container.
pub fn group() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
        }
    }
}

/// Group header component.
pub fn group_header() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(10.0)),
            min_height: px(30.0),
            column_gap: px(4.0),
            border_radius: BorderRadius::px(4.0, 4.0, 0.0, 0.0),
        }
        BackgroundColor(Color::srgb(0.22, 0.22, 0.25))
        BorderColor::all(Color::srgb(0.32, 0.32, 0.36))
    }
}

/// Group body container.
pub fn group_body() -> impl Scene {
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
        BackgroundColor(Color::srgb(0.16, 0.16, 0.19))
        BorderColor::all(Color::srgb(0.32, 0.32, 0.36))
    }
}
