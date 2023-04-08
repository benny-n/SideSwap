#![allow(clippy::type_complexity)]

use animation::AnimatorPlugin;
use bevy::{prelude::*, window::PrimaryWindow};
use effects::EffectsPlugin;
use events::EventPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use ui::UIPlugin;

mod animation;
mod effects;
mod events;
pub mod physics;
mod player;
mod ui;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum Wall {
    Left,
    Right,
}

#[derive(Resource)]
pub struct Score(pub usize);
#[derive(Resource)]
pub struct HighScore(pub usize);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(166, 234, 255)))
        .insert_resource(HighScore(0))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Jam #3".into(),
                resolution: (800., 600.).into(),
                canvas: Some("#bevy".into()),
                ..default()
            }),
            ..default()
        }))
        .add_state::<AppState>()
        .add_startup_system(spawn_camera)
        .add_system(reset_score.in_schedule(OnEnter(AppState::InGame)))
        .add_systems((update_highscore, exit_game).in_set(OnUpdate(AppState::InGame)))
        .add_plugin(UIPlugin)
        .add_plugin(EffectsPlugin)
        .add_plugin(EventPlugin)
        .add_plugin(AnimatorPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(PlayerPlugin)
        .run();
}

fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

fn reset_score(mut commands: Commands) {
    commands.insert_resource(Score(0));
}

fn update_highscore(score: Res<Score>, mut highscore: ResMut<HighScore>) {
    if score.0 > highscore.0 {
        highscore.0 = score.0;
    }
}

fn exit_game(keyboard_input: Res<Input<KeyCode>>, mut state: ResMut<NextState<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state.set(AppState::MainMenu);
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    YouDied,
}
