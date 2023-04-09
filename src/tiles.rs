use bevy::{prelude::*, utils::Instant, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use rand::{random, thread_rng, Rng};

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
#[derive(Component)]
pub struct FirstPlatform;

#[derive(Resource, DerefMut, Deref)]
pub struct PlatformTimer(pub Instant);

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_obstacles.in_schedule(OnEnter(AppState::InGame)))
            .add_systems(
                (
                    emit_platforms,
                    apply_icy_platforms,
                    despawn_out_of_screen_platforms.after(apply_icy_platforms),
                )
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
    let wall_width = 250.;
    let wall_height = window.height();

    // Ground
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(ground_width, ground_height)),
                ..default()
            },
            texture: asset_server.load("textures/lava.png"),
            transform: Transform::from_translation(Vec3::new(ground_width / 2., -20., 500.)),
            ..default()
        })
        .insert(Ground);

    // Walls
    [0., wall_height].into_iter().for_each(|y| {
        [(-110., Wall::Left), (110. + window.width(), Wall::Right)]
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
                        transform: Transform::from_xyz(x, y + wall_height / 2., 500.),
                        ..default()
                    })
                    .insert(ColliderMassProperties::Density(f32::INFINITY))
                    .insert(wall);
            })
    });

    // Starting platform
    commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(Collider::cuboid(
            PLATFORM_MIN_WIDTH / 2.,
            PLATFORM_SPRITE_SIZE / 2.,
        ))
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(PLATFORM_MIN_WIDTH, PLATFORM_SPRITE_SIZE)),
                ..default()
            },
            texture: asset_server.load("sprites/platform.png"),
            transform: Transform::from_xyz(PLATFORM_SPRITE_SIZE * 2., PLATFORM_MIN_Y, 1.),
            ..default()
        })
        .insert(Platform)
        .insert(FirstPlatform)
        .insert(Velocity::linear(Vec2::new(-10., 0.)));

    commands.insert_resource(PlatformTimer(Instant::now()));
}

const PLATFORM_START_WIDTH: f32 = 250.;
const PLATFORM_SPRITE_SIZE: f32 = 32.;
const PLATFORM_MIN_WIDTH: f32 = 3. * PLATFORM_SPRITE_SIZE;
const PLATFORM_MIN_Y: f32 = 150.;

fn emit_platforms(
    mut commands: Commands,
    timer: Res<PlatformTimer>,
    score: Res<Score>,
    effect_q: Res<EffectQueue>,
    asset_server: Res<AssetServer>,
    last_wall_query: Query<&LastWall>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let (plat_velocity, delta_secs) = if let Some(Effect::FastPlatforms) = effect_q.last() {
        (300., 0.75)
    } else {
        (150., 1.5)
    };
    // Spawn a platform every second
    if timer.elapsed().as_secs_f32() < delta_secs {
        return;
    }
    // Reset timer
    commands.insert_resource(PlatformTimer(Instant::now()));

    // Platforms should get smaller as the score increases
    let mut rng = thread_rng();
    let max_plat_parts = match score.0 {
        0..=3 => 4,
        4..=6 => 3,
        7..=9 => 2,
        _ => 1,
    };
    let plat_num = rng.gen_range(1..=max_plat_parts) as usize;

    let platform_height = PLATFORM_SPRITE_SIZE;
    let platform_y = PLATFORM_MIN_Y + random::<f32>() * (window.height() / 15.);
    let (platform_x, velocity) = match last_wall_query.get_single() {
        Ok(LastWall(Wall::Left)) => (
            window.width() + PLATFORM_START_WIDTH,
            Vec2::new(-(random::<f32>() * 5. + plat_velocity), 0.),
        ),
        Ok(LastWall(Wall::Right)) => (
            -PLATFORM_START_WIDTH,
            Vec2::new(random::<f32>() * 5. + plat_velocity, 0.),
        ),
        Err(_) => return,
    };

    // Apply side effects..
    let is_fallthrough =
        effect_q.last() == Some(&Effect::FallthroughPlatforms) && random::<f32>() < 0.25;

    let color = if is_fallthrough {
        Color::rgba(1., 1., 1., 0.5)
    } else if effect_q.last() == Some(&Effect::IcyPlatforms) {
        Color::rgb(0., 0.2, 0.9)
    } else {
        Color::WHITE
    };

    let atlas = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/platform.png"),
        Vec2::new(PLATFORM_SPRITE_SIZE, PLATFORM_SPRITE_SIZE),
        3,
        1,
        None,
        None,
    ));

    let spawn_platform_sprite = |parent: &mut ChildBuilder, index: usize, x: f32| {
        parent.spawn((
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index,
                    color,
                    ..default()
                },
                texture_atlas: atlas.clone(),
                transform: Transform::from_translation(Vec3::new(x, 0., 1.)),
                ..default()
            },
            Platform,
        ));
    };

    let entity = commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 1,
                custom_size: Some(Vec2::new(if plat_num == 1 { 0. } else { 32. }, 32.)),
                color,
                ..default()
            },
            texture_atlas: atlas.clone(),
            transform: Transform::from_translation(Vec3::new(
                platform_x,
                platform_y,
                1. + 1.5 * score.0 as f32,
            )),
            ..default()
        })
        .insert(Friction {
            coefficient: 0.,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Velocity::linear(velocity))
        .insert(Platform)
        .with_children(|parent| {
            for i in 1..plat_num {
                // spawn left side of the platform
                spawn_platform_sprite(parent, 1, -(PLATFORM_SPRITE_SIZE * i as f32));
                // spawn right side of the platform
                spawn_platform_sprite(parent, 1, PLATFORM_SPRITE_SIZE * i as f32);
            }
            let width = if plat_num == 1 {
                PLATFORM_SPRITE_SIZE / 2.
            } else {
                PLATFORM_SPRITE_SIZE
            };
            // spawn left side of the platform
            spawn_platform_sprite(parent, 0, -(width * plat_num as f32));
            // spawn right side of the platform
            spawn_platform_sprite(parent, 2, width * plat_num as f32);
        })
        .id();

    if !is_fallthrough {
        let collider_width = if plat_num == 1 {
            PLATFORM_SPRITE_SIZE * 2.
        } else {
            PLATFORM_MIN_WIDTH + ((plat_num - 1) as f32) * PLATFORM_SPRITE_SIZE * 2.
        };
        commands.entity(entity).insert(Collider::cuboid(
            (collider_width - PLATFORM_SPRITE_SIZE * 1.2) / 2.,
            (platform_height - 15.) / 2.,
        ));
    }
    if effect_q.last() == Some(&Effect::IcyPlatforms) {
        commands.entity(entity).insert(Icy);
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

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Icy;

fn apply_icy_platforms(
    effect_q: Res<EffectQueue>,
    mut commands: Commands,
    mut platform_query: Query<
        (Entity, Option<&mut Sprite>, Option<&mut TextureAtlasSprite>),
        With<Platform>,
    >,
) {
    if !effect_q.is_changed() {
        return;
    }
    let color = if let Some(Effect::IcyPlatforms) = effect_q.last() {
        Color::rgb(0.0, 0.2, 0.9)
    } else {
        Color::WHITE
    };
    for (entity, mut sprite, mut atlas_sprite) in platform_query.iter_mut() {
        if let Some(sprite) = sprite.as_mut() {
            if sprite.color.a() == 1. {
                sprite.color = color;
            }
        }
        if let Some(atlas_sprite) = atlas_sprite.as_mut() {
            if atlas_sprite.color.a() == 1. {
                atlas_sprite.color = color;
            }
        }
        if color == Color::WHITE {
            commands.entity(entity).remove::<Icy>();
        } else {
            commands.entity(entity).insert(Icy);
        }
    }
}

fn despawn_obstacles<T: Component>(mut commands: Commands, mut query: Query<Entity, With<T>>) {
    for entity in query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}
