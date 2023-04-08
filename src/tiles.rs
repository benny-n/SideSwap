use bevy::{prelude::*, utils::Instant, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use rand::random;

use crate::{
    effects::{Effect, EffectQueue},
    player::LastWall,
    AppState, Score, Wall,
};

pub struct TilesPlugin;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct Platform;

#[derive(Resource, DerefMut, Deref)]
pub struct PlatformTimer(pub Instant);

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_obstacles.in_schedule(OnEnter(AppState::InGame)))
            .add_systems(
                (emit_platforms, despawn_out_of_screen_platforms)
                    .in_set(OnUpdate(AppState::InGame)),
            )
            .add_systems(
                (
                    despawn_obstacles::<Wall>,
                    despawn_obstacles::<Ground>,
                    despawn_obstacles::<Platform>,
                )
                    .in_schedule(OnExit(AppState::InGame)),
            );
    }
}

fn spawn_obstacles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    let ground_width = window.width();
    let ground_height = 150.;
    let wall_width = 50.;
    let wall_height = window.height();

    // Ground
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(ground_width, ground_height)),
                ..default()
            },
            texture: asset_server.load("textures/lava.png"),
            transform: Transform::from_translation(Vec3::new(ground_width / 2., -20., 0.)),
            ..default()
        })
        .insert(Ground);

    // Walls
    [0., wall_height].into_iter().for_each(|y| {
        [(-10., Wall::Left), (10. + window.width(), Wall::Right)]
            .into_iter()
            .for_each(|(x, wall)| {
                commands
                    .spawn(RigidBody::Fixed)
                    .insert(Collider::cuboid(wall_width / 2., wall_height / 2.))
                    .insert(SpriteBundle {
                        sprite: Sprite {
                            // brownish
                            color: Color::rgb(0.5, 0.3, 0.1),
                            custom_size: Some(Vec2::new(wall_width, wall_height)),
                            ..default()
                        },
                        texture: asset_server.load("textures/brick.png"),
                        transform: Transform::from_xyz(x, y + wall_height / 2., 0.),
                        ..default()
                    })
                    .insert(wall);
            })
    });

    // Starting platform
    commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(Collider::cuboid(
            PLATFORM_START_WIDTH / 2.,
            PLATFORM_HEIGHT / 2.,
        ))
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(PLATFORM_START_WIDTH, PLATFORM_HEIGHT)),
                ..default()
            },
            texture: asset_server.load("textures/brick.png"),
            transform: Transform::from_xyz(10., PLATFORM_MIN_Y, 0.),
            ..default()
        })
        .insert(Platform)
        .insert(Velocity::linear(Vec2::new(-10., 0.)));

    commands.insert_resource(PlatformTimer(Instant::now()));
}

const PLATFORM_START_WIDTH: f32 = 250.;
const PLATFORM_MIN_WIDTH: f32 = 15.;
const PLATFORM_HEIGHT: f32 = 30.;
const PLATFORM_MIN_Y: f32 = 150.;

fn emit_platforms(
    mut commands: Commands,
    timer: Res<PlatformTimer>,
    score: Res<Score>,
    effect_q: Res<EffectQueue>,
    asset_server: Res<AssetServer>,
    last_wall_query: Query<&LastWall>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    // Spawn a platform every second
    if timer.elapsed().as_secs_f32() < 1.5 {
        return;
    }
    // Reset timer
    commands.insert_resource(PlatformTimer(Instant::now()));

    // Platforms should get smaller as the score increases
    let platform_base_width = PLATFORM_START_WIDTH - score.0 as f32 * 20.;
    let platform_rand_width =
        (random::<f32>() * 0.2 * platform_base_width) + 0.8 * platform_base_width;

    let platform_width = f32::max(PLATFORM_MIN_WIDTH, platform_rand_width);
    let platform_height = PLATFORM_HEIGHT;
    let platform_y = PLATFORM_MIN_Y + random::<f32>() * (window.height() / 15.);
    let (platform_x, velocity) = match last_wall_query.get_single() {
        Ok(LastWall(Wall::Left)) => (
            window.width() + PLATFORM_START_WIDTH,
            Vec2::new(-(random::<f32>() * 0. + 150.), 0.),
        ),
        Ok(LastWall(Wall::Right)) => (
            -PLATFORM_START_WIDTH,
            Vec2::new(random::<f32>() * 0. + 150., 0.),
        ),
        Err(_) => return,
    };

    // Apply side effects..
    let is_tranparent =
        effect_q.last() == Some(&Effect::TransparentPlatforms) && random::<f32>() < 0.25;

    let entity = commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(platform_width * 0.8, platform_height)),
                color: if is_tranparent {
                    Color::rgba(1., 1., 1., 0.5)
                } else {
                    Color::WHITE
                },
                ..default()
            },
            texture: asset_server.load("textures/brick.png"),
            transform: Transform::from_translation(Vec3::new(platform_x, platform_y, 0.)),
            ..default()
        })
        .insert(Friction {
            coefficient: 0.,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Velocity::linear(velocity))
        .insert(Platform)
        .with_children(|parent| {
            // spawn left side of the platform
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(platform_width * 0.1, platform_height)),
                    color: Color::BLUE,
                    ..default()
                },
                texture: asset_server.load("textures/brick.png"),
                transform: Transform::from_translation(Vec3::new(-platform_width * 0.4, 0., -1.)),
                ..default()
            });
            // spawn right side of the platform
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(platform_width * 0.1, platform_height)),
                    color: Color::RED,
                    ..default()
                },
                texture: asset_server.load("textures/brick.png"),
                transform: Transform::from_translation(Vec3::new(platform_width * 0.4, 0., -1.)),
                ..default()
            });
        })
        .id();

    if !is_tranparent {
        commands
            .entity(entity)
            .insert(Collider::cuboid(platform_width / 2., platform_height / 2.));
    }
}

fn despawn_out_of_screen_platforms(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Platform>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    for (entity, transform) in query.iter() {
        if transform.translation.x < -500. || transform.translation.x > window.width() + 500. {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn despawn_obstacles<T: Component>(mut commands: Commands, mut query: Query<Entity, With<T>>) {
    for entity in query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}
