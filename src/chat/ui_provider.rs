use bevy::{prelude::*, ui::RelativeCursorPosition};
use crate::chat::controller::{NameBoxBackground, NameText, TextBoxBackground};

use bevy::{color::palettes::css::RED, prelude::*, ui::RelativeCursorPosition};

use crate::chat::{controller::{InfoText, MessageText, NameBoxBackground, NameText, TextBoxBackground}, GUIScrollText};
pub fn backplate_container() -> impl Bundle {
    (
        Node {
            width: Val::Vw(70.),
            height: Val::Percent(20.),
            margin: UiRect::all(Val::Auto).with_bottom(Val::Px(45.)),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )
}

pub fn top_section() -> impl Bundle {
    // Needed for horizontal flex,
    // open to modification
    Node::default()
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
        GlobalZIndex(3),
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
        GlobalZIndex(3),
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
