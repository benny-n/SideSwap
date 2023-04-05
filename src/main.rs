#![allow(clippy::type_complexity)]

use animation::AnimatorPlugin;
use bevy::{prelude::*, window::PrimaryWindow};
use player::PlayerPlugin;

mod animation;
mod player;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Jam #3".into(),
                resolution: (800., 600.).into(),
                canvas: Some("#bevy".into()),
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(spawn_camera)
        .add_state::<AppState>()
        .add_plugin(ui::UIPlugin)
        .add_plugin(AnimatorPlugin)
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

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    YouDied,
}
