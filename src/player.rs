use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{Animation, AnimationTimer, Animations},
    events::WallReached,
    AppState, Wall,
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
                    read_result_system,
                    modify_character_controller_slopes, // confine_player_in_screen,
                                                        // send_wall_reached_event,
                )
                    .in_set(OnUpdate(AppState::InGame)),
            )
            .add_system(despawn_player.in_schedule(OnExit(AppState::InGame)));
    }
}

/* Read the character controller collisions stored in the character controller’s output. */
fn modify_character_controller_slopes(
    mut character_controller_outputs: Query<&mut KinematicCharacterControllerOutput>,
) {
    for mut output in character_controller_outputs.iter_mut() {
        for collision in &output.collisions {
            // Do something with that collision information.
            info!("Collision: {:?}", collision);
        }
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
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::capsule_y(
            HALF_PLAYER_SIZE / 2.,
            HALF_PLAYER_SIZE / 2.,
        ))
        .insert(KinematicCharacterController::default())
        .insert(AnimationTimer(Timer::from_seconds(
            1. / idle.fps as f32,
            TimerMode::Repeating,
        )));
    commands.insert_resource(Animations {
        map: animations,
        active: idle,
    });
}

fn move_player(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut KinematicCharacterController,
        ),
        With<Player>,
    >,
) {
    for (mut velocity, mut transform, mut controller) in query.iter_mut() {
        velocity.x = velocity.x.clamp(-PLAYER_SPEED, PLAYER_SPEED);
        let translation = Vec3::new(
            velocity.x * time.delta_seconds(),
            velocity.y * time.delta_seconds(),
            0.,
        );
        transform.translation += translation;
        controller.translation = Some(Vec2::new(
            controller.translation.unwrap_or_default().x + translation.x,
            controller.translation.unwrap_or_default().y + translation.y,
        ));
    }
}

fn read_result_system(controllers: Query<(Entity, &KinematicCharacterControllerOutput)>) {
    for (entity, output) in controllers.iter() {
        // info!(
        //     "Entity {:?} moved by {:?} and touches the ground: {:?}",
        //     entity, output.effective_translation, output.grounded
        // );
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
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Transform, Option<&Wall>), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    let Ok((player, mut player_transform, wall)) = player_query.get_single_mut() else {
        return;
    };

    let x_min = 0.0 + HALF_PLAYER_SIZE / 2.;
    let x_max = window.width() - HALF_PLAYER_SIZE / 2.;
    let y_min = 0.0 + HALF_PLAYER_SIZE;
    let y_max = window.height() - HALF_PLAYER_SIZE;

    let mut translation = player_transform.translation;

    translation.x = translation.x.clamp(x_min, x_max);
    translation.y = translation.y.clamp(y_min, y_max);

    player_transform.translation = translation;

    // Insert a wall component if the player is at the edge of the screen
    // This is used to send a WallReached event
    if translation.x == x_min && wall != Some(&Wall::Left) {
        commands.entity(player).insert(Wall::Left);
    } else if translation.x == x_max && wall != Some(&Wall::Right) {
        commands.entity(player).insert(Wall::Right);
    }
}

fn send_wall_reached_event(
    mut wall_reached_events: EventWriter<WallReached>,
    mut query: Query<(&Wall, Changed<Wall>), With<Player>>,
) {
    // only send the event if the wall that was reached is new
    if let Ok((wall, reached_new_wall)) = query.get_single_mut() {
        if reached_new_wall {
            wall_reached_events.send(WallReached(*wall));
        }
    };
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
