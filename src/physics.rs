use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;

use crate::AppState;

pub struct PhysicsPlugin;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct Wall;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_obstacles.in_schedule(OnEnter(AppState::InGame)))
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugin(RapierDebugRenderPlugin::default()) // TODO: remove this
            .add_systems(
                (despawn_obstacles::<Wall>, despawn_obstacles::<Ground>)
                    .in_schedule(OnExit(AppState::InGame)),
            );
    }
}

fn spawn_obstacles(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    let ground_width = window.width();
    let ground_height = 15.;
    let wall_width = 15.;
    let wall_height = window.height();

    // Ground
    commands
        .spawn(RigidBody::Fixed)
        .insert(Collider::cuboid(ground_width / 2., ground_height / 4.))
        .insert(SpriteBundle {
            sprite: Sprite {
                // brownish
                color: Color::rgb(0.5, 0.3, 0.1),
                custom_size: Some(Vec2::new(ground_width, ground_height)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(ground_width / 2., 0., 0.)),
            ..default()
        })
        .insert(Ground);

    // Walls
    [0., window.width()].into_iter().for_each(|x| {
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
                transform: Transform::from_translation(Vec3::new(x, wall_height / 2., 0.)),
                ..default()
            })
            .insert(Wall);
    });
}

fn despawn_obstacles<T: Component>(mut commands: Commands, mut query: Query<Entity, With<T>>) {
    for entity in query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}
