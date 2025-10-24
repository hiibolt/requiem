use bevy::{color::palettes::css::RED, prelude::*, ui::RelativeCursorPosition};

use crate::chat::{controller::{InfoText, MessageText, NameBoxBackground, NameText, TextBoxBackground}, GUIScrollText};
pub fn backplate_container() -> impl Bundle {
    (
        Node {
            width: Val::Vw(70.),
            height: Val::Vh(20.),
            margin: UiRect::all(Val::Auto).with_bottom(Val::Px(25.)),
            // position_type: PositionType::Absolute,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ZIndex(1),
    )
}

pub fn top_section() -> impl Bundle {
    Node {
        min_height: Val::Px(85.),
        ..default()
    }
}

pub fn namebox() -> impl Bundle {
    (
        ImageNode::default(),
        Node {
            margin: UiRect::default().with_left(Val::Px(45.)),
            aspect_ratio: Some(3.),
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Visible,
        ZIndex(2),
        NameBoxBackground,
    )
}

pub fn nametext(asset_server: &Res<AssetServer>) -> impl Bundle {
    (
        Node {
            margin: UiRect::default().with_left(Val::Px(35.)),
            ..default()
        },
        Text::new("TEST"),
        TextFont {
            font: asset_server.load("fonts/ALLER.ttf"),
            font_size: 30.0,
            ..default()
        },
        ZIndex(3),
        NameText
    )
}

pub fn textbox() -> impl Bundle {
    (
        ImageNode::default(),
        Node {
            width: Val::Percent(100.),
            min_height: Val::Percent(100.),
            ..default()
        },
        ZIndex(2),
        Visibility::Visible,
        RelativeCursorPosition::default(),
        TextBoxBackground,
    )
}

pub fn messagetext(asset_server: &Res<AssetServer>) -> impl Bundle {
    (
        Text::new("TEST"),
        GUIScrollText::default(),
        Node::default(),
        TextFont {
            font: asset_server.load("fonts/ALLER.ttf"),
            font_size: 40.0,
            ..default()
        },
        ZIndex(3),
        MessageText
    )
}

pub fn infotext(asset_server: &Res<AssetServer>) -> impl Bundle {
    (
        Text::new("TEST"),
        Node::default(),
        TextFont {
            font: asset_server.load("fonts/ALLER.ttf"),
            font_size: 40.0,
            ..default()
        },
        TextLayout {
            justify: Justify::Center,
            linebreak: LineBreak::WordBoundary,
        },
        TextColor(Color::Srgba(RED)),
        ZIndex(3),
        InfoText
    )
}