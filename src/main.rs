use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Jam #3".into(),
                resolution: (800., 600.).into(),
                canvas: Some("#bevy".into()),
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(setup_menu)
        .run();
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "Hello, World",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::ORANGE_RED,
            },
        )
        .with_alignment(TextAlignment::Center),
        ..default()
    });
}
