use bevy::prelude::*;

use crate::AppState;

#[derive(Component)]
pub struct MainMenu;

const TITLE_TEXT: &str = "SideSwap";
const DESCRIPTION_TEXT: &str = "Side effects, literally.";
const TUTORIAL_TEXT: &str =
    "Use [A, D] to move.\n Press SPACE to jump.\n Get to the other side of the screen!";

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

#[derive(Resource)]
pub struct ButtonColors {
    default: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            default: Color::CRIMSON,
            hovered: {
                let [r, g, b, _] = Color::CRIMSON.as_rgba_f32();
                Color::rgba(r, g, b, 0.5)
            },
        }
    }
}

pub fn click_play_button(
    button_colors: Res<ButtonColors>,
    mut state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.default.into();
            }
        }
    }
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
