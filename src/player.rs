use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};

use crate::{
    animation::{Animation, AnimationTimer, Animations},
    AppState,
};

const PLAYER_SPEED: f32 = 500.;
const PLAYER_ACCELERATION: f32 = PLAYER_SPEED / 2.;
const JUMP_VELOCITY: f32 = 1750.;
const PLAYER_SIZE: f32 = 150.;
const HALF_PLAYER_SIZE: f32 = PLAYER_SIZE / 2.;
const FRICTION: f32 = PLAYER_ACCELERATION * 2.;
const GRAVITY: f32 = 250.;

#[derive(Component)]
struct Player;

#[derive(Component, Default, Debug, Clone, Copy)]
pub enum Facing {
    #[default]
    Right,
    Left,
}

#[derive(Component)]
struct Midair;

#[derive(Component, DerefMut, Deref)]
struct Velocity(Vec2);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_player.in_schedule(OnEnter(AppState::InGame)))
            .add_systems(
                (
                    player_input,
                    gravity,
                    land_on_ground,
                    change_player_animation,
                    move_player,
                    confine_player_in_screen,
                )
                    .in_set(OnUpdate(AppState::InGame)),
            )
            .add_system(despawn_player.in_schedule(OnExit(AppState::InGame)));
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let animations = HashMap::from_iter(vec![
        (
            "idle".to_owned(),
            Animation {
                handle: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("sprites/idle.png"),
                    Vec2::new(100.0, 150.0),
                    12,
                    1,
                    None,
                    None,
                )),
                last: 11,
                curr: 0,
                fps: 12,
            },
        ),
        (
            "run".to_owned(),
            Animation {
                handle: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("sprites/run.png"),
                    Vec2::new(150.0, 150.0),
                    18,
                    1,
                    None,
                    None,
                )),
                last: 17,
                curr: 0,
                fps: 24,
            },
        ),
        (
            "jump".to_owned(),
            Animation {
                handle: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("sprites/jump.png"),
                    Vec2::new(150.0, 150.0),
                    1,
                    1,
                    None,
                    None,
                )),
                last: 1,
                curr: 0,
                fps: 1,
            },
        ),
    ]);

    let idle = animations["idle"].clone();

    commands
        .spawn(Player)
        .insert(Velocity(Vec2::new(0., 0.)))
        .insert(Facing::default())
        .insert(SpriteSheetBundle {
            texture_atlas: idle.handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(HALF_PLAYER_SIZE, HALF_PLAYER_SIZE, 0.0),
            ..default()
        })
        .insert(AnimationTimer(Timer::from_seconds(
            1. / idle.fps as f32,
            TimerMode::Repeating,
        )));
    commands.insert_resource(Animations {
        map: animations,
        active: idle,
    });
}

fn move_player(time: Res<Time>, mut query: Query<(&mut Velocity, &mut Transform), With<Player>>) {
    for (mut velocity, mut transform) in query.iter_mut() {
        velocity.x = velocity.x.clamp(-PLAYER_SPEED, PLAYER_SPEED);
        transform.translation += Vec3::new(
            velocity.x * time.delta_seconds(),
            velocity.y * time.delta_seconds(),
            0.,
        );
    }
}

fn player_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut Velocity, &mut Facing, Option<&Midair>), With<Player>>,
) {
    for (player, mut velocity, mut facing, optionally_midair) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            velocity.x -= PLAYER_ACCELERATION;
            *facing = Facing::Left;
        } else if keyboard_input.pressed(KeyCode::D) {
            velocity.x += PLAYER_ACCELERATION;
            *facing = Facing::Right;
        } else {
            let delta = f32::min(velocity.x.abs(), FRICTION);
            velocity.x -= velocity.x.signum() * delta;
        }
        if keyboard_input.pressed(KeyCode::Space) && optionally_midair.is_none() {
            velocity.y += JUMP_VELOCITY;
            commands.entity(player).insert(Midair);
        }
    }
}

fn confine_player_in_screen(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let (Ok(mut player_transform), Ok(window)) = (player_query.get_single_mut(), window_query.get_single()) else {
        return;
    };

    let x_min = 0.0 + HALF_PLAYER_SIZE / 2.;
    let x_max = window.width() - HALF_PLAYER_SIZE / 2.;
    let y_min = 0.0 + HALF_PLAYER_SIZE;
    let y_max = window.height() - HALF_PLAYER_SIZE;

    let mut translation = player_transform.translation;

    // Bound the player x position
    if translation.x < x_min {
        translation.x = x_min;
    } else if translation.x > x_max {
        translation.x = x_max;
    }

    // Bound the player y position
    if translation.y < y_min {
        translation.y = y_min;
    } else if translation.y > y_max {
        translation.y = y_max;
    }

    player_transform.translation = translation;
}

fn change_player_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &Velocity, Option<&Midair>), With<Player>>,
    mut animations: ResMut<Animations>,
) {
    for (player, velocity, optionally_midair) in query.iter_mut() {
        let old_animation_handle = animations.active.handle.clone();
        animations.active = if optionally_midair.is_some() {
            &animations.map["jump"]
        } else if velocity.x == 0. {
            &animations.map["idle"]
        } else {
            &animations.map["run"]
        }
        .clone();
        if animations.active.handle == old_animation_handle {
            return;
        }
        commands.entity(player).insert((
            animations.active.handle.clone(),
            TextureAtlasSprite::new(animations.active.curr),
            AnimationTimer(Timer::from_seconds(
                1. / animations.active.fps as f32,
                TimerMode::Repeating,
            )),
        ));
    }
}

fn gravity(mut query: Query<&mut Velocity, With<Midair>>) {
    for mut velocity in query.iter_mut() {
        velocity.y -= GRAVITY;
    }
}

fn land_on_ground(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Velocity), With<Midair>>,
) {
    for (player, transform, mut velocity) in query.iter_mut() {
        if transform.translation.y - HALF_PLAYER_SIZE <= 0. {
            velocity.y = 0.;
            commands.entity(player).remove::<Midair>();
        }
    }
}

fn despawn_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
