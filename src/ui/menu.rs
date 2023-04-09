use bevy::prelude::*;

use super::ButtonColors;

#[derive(Component)]
pub struct MainMenu;

const TITLE_TEXT: &str = "SideSwap";
const DESCRIPTION_TEXT: &str = "Side effects, literally.";
const TUTORIAL_TEXT: &str =
    "Use [A, D] to move.\n Press SPACE to jump.\n Get to the wall highlighted in red!";

pub fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                ..default()
            },
            MainMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((TextBundle {
                        style: Style {
                            margin: UiRect::bottom(Val::Px(32.)),
                            ..default()
                        },
                        text: Text {
                            sections: vec![TextSection::new(
                                TITLE_TEXT,
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 80.0,
                                    color: Color::CRIMSON,
                                },
                            )],
                            alignment: TextAlignment::Center,
                            ..default()
                        },
                        ..default()
                    },));
                    parent.spawn((TextBundle {
                        style: Style {
                            margin: UiRect::bottom(Val::Px(8.)),
                            ..default()
                        },
                        text: Text {
                            sections: vec![TextSection::new(
                                DESCRIPTION_TEXT,
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 40.0,
                                    color: Color::ORANGE_RED,
                                },
                            )],
                            alignment: TextAlignment::Center,
                            ..default()
                        },
                        ..default()
                    },));
                    parent.spawn((TextBundle {
                        style: Style {
                            justify_content: JustifyContent::FlexEnd,
                            ..default()
                        },
                        text: Text {
                            sections: vec![TextSection::new(
                                TUTORIAL_TEXT,
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 24.0,
                                    color: Color::BLACK,
                                },
                            )],
                            alignment: TextAlignment::Center,
                            ..default()
                        },
                        ..default()
                    },));
                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                align_self: AlignSelf::Center,
                                max_size: Size::new(Val::Px(250.), Val::Auto),
                                margin: UiRect::all(Val::Px(8.)),
                                ..default()
                            },
                            background_color: BackgroundColor(ButtonColors::default().default),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Let's GOOOO!",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });
        });
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
