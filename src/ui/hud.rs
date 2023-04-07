use bevy::{prelude::*, utils::Instant};

use crate::Score;

const TIME_SECTION: usize = 1;
const SCORE_SECTION: usize = 1;
const DIRECTION_SECTION: usize = 0;
#[derive(Component)]
pub struct Hud;
#[derive(Component)]
pub struct DirectionText;

const fn hud_text_style(font: Handle<Font>) -> TextStyle {
    TextStyle {
        font,
        font_size: 48.0,
        color: Color::CRIMSON,
    }
}

const GO_LEFT_TEXT: &str = "<<<";
const GO_RIGHT_TEXT: &str = ">>>";

#[derive(Resource, Deref, DerefMut)]
pub struct Timer(pub Instant);

#[derive(Component)]
pub struct TimerText;

#[derive(Component)]
pub struct ScoreText;

pub fn update_timer(timer: Res<Timer>, mut query: Query<&mut Text, With<TimerText>>) {
    let Ok(mut text) = query.get_single_mut() else {
        return;
    };
    let elapsed = timer.0.elapsed().as_secs_f32();
    text.sections[TIME_SECTION].value = format!("{:.2}", elapsed);
}

pub fn start_timer(mut commands: Commands) {
    commands.insert_resource(Timer(Instant::now()));
}

pub fn update_score(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        let Ok(mut text) = query.get_single_mut() else {
            return;
        };
        text.sections[SCORE_SECTION].value = score.0.to_string();
    }
}

pub fn update_direction(score: Res<Score>, mut query: Query<&mut Text, With<DirectionText>>) {
    if score.is_changed() {
        let Ok(mut text) = query.get_single_mut() else {
            return;
        };
        match text.sections[DIRECTION_SECTION].value.as_str() {
            GO_LEFT_TEXT => text.sections[DIRECTION_SECTION].value = GO_RIGHT_TEXT.to_string(),
            GO_RIGHT_TEXT => text.sections[DIRECTION_SECTION].value = GO_LEFT_TEXT.to_string(),
            _ => {}
        }
    }
}

pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    size: Size::new(Val::Percent(100.), Val::Auto),
                    // align_content: AlignContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            Hud,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    style: Style {
                        margin: UiRect::horizontal(Val::Px(16.)),
                        ..default()
                    },
                    text: Text {
                        sections: vec![
                            TextSection::new("Timer: ", hud_text_style(font.clone())),
                            TextSection::new(format!("{:.2}", 0.0), hud_text_style(font.clone())),
                        ],
                        ..default()
                    },
                    ..default()
                },
                TimerText,
            ));
            parent.spawn((
                TextBundle {
                    style: Style {
                        margin: UiRect::horizontal(Val::Px(16.)),
                        ..default()
                    },
                    text: Text {
                        sections: vec![TextSection::new(">>>", hud_text_style(font.clone()))],
                        ..default()
                    },
                    ..default()
                },
                DirectionText,
            ));
            parent.spawn((
                TextBundle {
                    style: Style {
                        margin: UiRect::horizontal(Val::Px(16.)),
                        ..default()
                    },
                    text: Text {
                        sections: vec![
                            TextSection::new("Score: ", hud_text_style(font.clone())),
                            TextSection::new("0", hud_text_style(font.clone())),
                        ],
                        ..default()
                    },
                    ..default()
                },
                ScoreText,
            ));
        });
}

pub fn despawn_hud(mut commands: Commands, query: Query<Entity, With<Hud>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
