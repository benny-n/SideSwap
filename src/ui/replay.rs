use bevy::prelude::*;

use crate::{events::Reason, HighScore, Score};

use super::ButtonColors;

#[derive(Component)]
pub struct ReplayScreen;

const YOU_DIED_TEXT: &str = "YOU DIED!";
const OUT_OF_TIME_TEXT: &str = "OUT OF TIME!";

pub fn spawn_replay_screen(
    mut commands: Commands,
    score: Res<Score>,
    highscore: Res<HighScore>,
    asset_server: Res<AssetServer>,
    loss_reason: Res<Reason>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let title = match *loss_reason {
        Reason::OutOfTime => OUT_OF_TIME_TEXT,
        Reason::Died => YOU_DIED_TEXT,
    };
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
            ReplayScreen,
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
                            margin: UiRect::vertical(Val::Px(18.)),
                            ..default()
                        },
                        text: Text {
                            sections: vec![TextSection::new(
                                title,
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 140.,
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
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        text: Text {
                            sections: vec![
                                TextSection::new(
                                    "Score: ",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: 40.0,
                                        color: Color::ORANGE_RED,
                                    },
                                ),
                                TextSection::new(
                                    score.0.to_string(),
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: 50.0,
                                        color: Color::RED,
                                    },
                                ),
                            ],
                            alignment: TextAlignment::Center,
                            ..default()
                        },
                        ..default()
                    },));
                    parent.spawn((TextBundle {
                        style: Style {
                            margin: UiRect::bottom(Val::Px(8.)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        text: Text {
                            sections: vec![
                                TextSection::new(
                                    "Highest: ",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: 40.0,
                                        color: Color::ORANGE_RED,
                                    },
                                ),
                                TextSection::new(
                                    highscore.0.to_string(),
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: 50.0,
                                        color: Color::RED,
                                    },
                                ),
                            ],
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
                                max_size: Size::new(Val::Px(275.), Val::Auto),
                                margin: UiRect::all(Val::Px(8.)),
                                ..default()
                            },
                            background_color: BackgroundColor(ButtonColors::default().default),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Again >:)",
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

pub fn despawn_replay_screen(mut commands: Commands, query: Query<Entity, With<ReplayScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
