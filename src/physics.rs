use bevy::{ecs::system::SystemParam, prelude::*, utils::Instant, window::PrimaryWindow};
use bevy_rapier2d::{na::Vector, prelude::*, rapier::prelude::PhysicsHooks};
use rand::random;

use crate::{player::LastWall, AppState, Wall};

pub struct PhysicsPlugin;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct Platform;

#[derive(Resource, DerefMut, Deref)]
pub struct PlatformTimer(pub Instant);

#[derive(SystemParam)]
struct CustomCollisionHook;

impl BevyPhysicsHooks for CustomCollisionHook {
    fn modify_solver_contacts(&self, context: ContactModificationContextView<'_, '_>) {
        let allowed_normal = -Vector::y();
        context.raw.update_as_oneway_platform(&allowed_normal, 0.);
    }
}

#[derive(Component, DerefMut, Deref)]
pub struct PlatformVelocity(pub f32);

#[derive(Resource)]
pub struct PhysicsHooksResource(Box<dyn PhysicsHooks + Send + Sync>);

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_obstacles.in_schedule(OnEnter(AppState::InGame)))
            .add_plugin(RapierPhysicsPlugin::<CustomCollisionHook>::pixels_per_meter(100.0))
            // .add_plugin(RapierDebugRenderPlugin::default()) // TODO: remove this
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
        // .spawn(RigidBody::Fixed)
        // .spawn(Collider::cuboid(ground_width / 2., ground_height / 2.))
        .spawn(SpriteBundle {
            sprite: Sprite {
                // brownish
                custom_size: Some(Vec2::new(ground_width, ground_height)),
                ..default()
            },
            texture: asset_server.load("textures/lava.png"),
            transform: Transform::from_translation(Vec3::new(ground_width / 2., -20., 0.)),
            ..default()
        })
        .insert(Ground);

    // Walls
    [(-10., Wall::Left), (10. + window.width(), Wall::Right)]
        .into_iter()
        .for_each(|(x, wall)| {
            commands
                .spawn(RigidBody::Fixed)
                .insert(Collider::cuboid(wall_width, wall_height / 2.))
                .insert(SpriteBundle {
                    sprite: Sprite {
                        // brownish
                        color: Color::rgb(0.5, 0.3, 0.1),
                        custom_size: Some(Vec2::new(wall_width, wall_height)),
                        ..default()
                    },
                    texture: asset_server.load("textures/brick.png"),
                    transform: Transform::from_xyz(x, wall_height / 2., 0.),
                    ..default()
                })
                .insert(wall);
        });

    // Starting platform
    commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(Collider::cuboid(
            PLATFORM_MIN_WIDTH / 2.,
            PLATFORM_MIN_HEIGHT / 2.,
        ))
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(PLATFORM_MIN_WIDTH, PLATFORM_MIN_HEIGHT)),
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

const PLATFORM_MIN_WIDTH: f32 = 250.;
const PLATFORM_MIN_HEIGHT: f32 = 20.;
const PLATFORM_MIN_Y: f32 = 150.;

fn emit_platforms(
    mut commands: Commands,
    timer: Res<PlatformTimer>,
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

    let platform_width = random::<f32>() * PLATFORM_MIN_WIDTH + 50.;
    let platform_height = PLATFORM_MIN_HEIGHT;
    let platform_y = PLATFORM_MIN_Y + random::<f32>() * (window.height() / 15.);
    let (platform_x, velocity) = match last_wall_query.get_single() {
        Ok(LastWall(Wall::Left)) => (
            window.width() + PLATFORM_MIN_WIDTH,
            Vec2::new(-(random::<f32>() * 0. + 150.), 0.),
        ),
        Ok(LastWall(Wall::Right)) => (
            -PLATFORM_MIN_WIDTH,
            Vec2::new(random::<f32>() * 0. + 150., 0.),
        ),
        Err(_) => return,
    };

    commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(Collider::cuboid(platform_width / 2., platform_height / 2.))
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(platform_width, platform_height)),
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
        .insert(ActiveHooks::MODIFY_SOLVER_CONTACTS)
        .insert(Platform);
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
