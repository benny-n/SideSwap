use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};

use crate::animation::{animate_sprites, Animation, AnimationTimer, Animations};

const PLAYER_SPEED: f32 = 500.0;
const PLAYER_SIZE: f32 = 150.0;
const HALF_PLAYER_SIZE: f32 = PLAYER_SIZE / 2.0;

#[derive(Component)]
struct Player;
#[derive(Component, Default, Debug, Clone, Copy)]
enum PlayerState {
    #[default]
    Idle,
    Running,
    // Jumping,
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub enum Facing {
    #[default]
    Right,
    Left,
}

#[derive(Component)]
struct PlayerMovement {
    x_velocity: f32,
    y_velocity: f32,
    direction: Vec2,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(move_player)
            .add_system(change_player_animation.before(animate_sprites))
            .add_system(player_input)
            .add_system(confine_player_movement.after(move_player));
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
    ]);

    let idle = animations["idle"].clone();

    commands
        .spawn(Player)
        .insert(PlayerState::default())
        .insert(PlayerMovement {
            x_velocity: PLAYER_SPEED,
            y_velocity: 0.,
            direction: Vec2::new(1., 0.),
        })
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

fn move_player(time: Res<Time>, mut query: Query<(&PlayerMovement, &mut Transform), With<Player>>) {
    for (movement, mut transform) in query.iter_mut() {
        transform.translation += Vec3::new(
            movement.direction.x * movement.x_velocity * time.delta_seconds(),
            movement.direction.y * movement.y_velocity * time.delta_seconds(),
            0.,
        );
    }
}

fn player_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut movement_query: Query<(Entity, &mut PlayerMovement, &PlayerState), With<Player>>,
    mut facing_query: Query<&mut Facing, With<Player>>,
) {
    let Ok(mut facing) = facing_query.get_single_mut() else {
        return;
    };
    let mut player_movement = Vec2::new(0., 0.);
    if keyboard_input.pressed(KeyCode::A) {
        player_movement.x -= 1.;
        *facing = Facing::Left;
    }
    if keyboard_input.pressed(KeyCode::D) {
        player_movement.x += 1.;
        *facing = Facing::Right;
    }
    for (player, mut movement, state) in movement_query.iter_mut() {
        movement.direction = player_movement.normalize_or_zero();
        if player_movement.x != 0. {
            if !matches!(state, PlayerState::Running) {
                commands.entity(player).insert(PlayerState::Running);
            }
        } else if !matches!(state, PlayerState::Idle) {
            commands.entity(player).insert(PlayerState::Idle);
        }
    }
}

fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let (Ok(mut player_transform), Ok(window)) = (player_query.get_single_mut(), window_query.get_single()) else {
        return;
    };

    let x_min = 0.0 + HALF_PLAYER_SIZE / 2.;
    let x_max = window.width() - HALF_PLAYER_SIZE / 2.;

    let mut translation = player_transform.translation;

    // Bound the player x position
    if translation.x < x_min {
        translation.x = x_min;
    } else if translation.x > x_max {
        translation.x = x_max;
    }

    player_transform.translation = translation;
}

fn change_player_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &PlayerState, Changed<PlayerState>), With<Player>>,
    mut animations: ResMut<Animations>,
) {
    for (player, state, changed) in query.iter_mut() {
        if !changed {
            continue;
        }
        animations.active = match state {
            PlayerState::Idle => &animations.map["idle"],
            PlayerState::Running => &animations.map["run"],
        }
        .clone();
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
