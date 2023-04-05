use bevy::{prelude::*, utils::Instant};

use crate::events::WallReached;

const TIME_SECTION: usize = 1;
const SCORE_SECTION: usize = 1;
#[derive(Component)]
pub struct Hud;

const fn hud_text_style(font: Handle<Font>) -> TextStyle {
    TextStyle {
        font,
        font_size: 48.0,
        color: Color::CRIMSON,
    }
}

const HUD_TOP_LEFT: Style = Style {
    position_type: PositionType::Absolute,
    position: UiRect::new(Val::Px(8.), Val::Auto, Val::Px(0.), Val::Auto),
    ..Style::DEFAULT
};

const HUD_TOP_RIGHT: Style = Style {
    position_type: PositionType::Absolute,
    position: UiRect::new(Val::Px(600.), Val::Auto, Val::Px(0.), Val::Auto),
    ..Style::DEFAULT
};

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

pub fn update_score(
    mut wall_reached_events: EventReader<WallReached>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    let Ok(mut text) = query.get_single_mut() else {
        return;
    };
    if wall_reached_events.iter().count() > 0 {
        let score = text.sections[1].value.parse::<i32>().unwrap_or(0) + 1;
        text.sections[SCORE_SECTION].value = score.to_string();
    }
}

pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    justify_content: JustifyContent::SpaceBetween,
                    align_content: AlignContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            Hud,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    style: HUD_TOP_LEFT,
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
                    style: HUD_TOP_RIGHT,
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
