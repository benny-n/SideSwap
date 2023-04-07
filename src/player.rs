use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{Animation, AnimationTimer, Animations},
    events::WallReached,
    physics::Platform,
    AppState, Wall,
};

const PLAYER_SPEED: f32 = 100.;
const JUMP_VELOCITY: f32 = 200.;
const PLAYER_SIZE: f32 = 150.;
const HALF_PLAYER_SIZE: f32 = PLAYER_SIZE / 2.;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default, Debug, Clone, Copy)]
pub enum Facing {
    #[default]
    Right,
    Left,
}

#[derive(Component, DerefMut, Deref)]
struct OnPlatform(pub Velocity);

#[derive(Component, DerefMut, Deref)]
pub struct LastWall(pub Wall);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_player.in_schedule(OnEnter(AppState::InGame)))
            .add_systems(
                (
                    handle_player_collisions,
                    player_input.after(handle_player_collisions),
                    change_player_animation,
                    send_wall_reached_event,
                    move_player,
                    // debug_velocity.after(player_input).after(move_player),
                )
                    .in_set(OnUpdate(AppState::InGame)),
            )
            .add_system(move_player.in_base_set(PhysicsSet::StepSimulation))
            // .add_system(debug_velocity.in_base_set(CoreSet::PostUpdate))
            .add_system(despawn_player.in_schedule(OnExit(AppState::InGame)));
    }
}

fn debug_velocity(mut query: Query<&Velocity, With<Player>>) {
    for velocity in &mut query {
        info!("DEBUG velocity: {:?}", velocity.linvel.x);
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
        .insert(Facing::default())
        .insert(SpriteSheetBundle {
            texture_atlas: idle.handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(HALF_PLAYER_SIZE, HALF_PLAYER_SIZE + 10., 0.),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        // .insert(ActiveHooks::MODIFY_SOLVER_CONTACTS)
        .insert(GravityScale(5.))
        .insert(Velocity {
            linvel: Vec2::new(0., 0.),
            angvel: 0.,
        })
        .insert(Restitution {
            coefficient: 0.,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Collider::cuboid(HALF_PLAYER_SIZE / 2., HALF_PLAYER_SIZE))
        .insert(KinematicCharacterController {
            // snap_to_ground: None,
            // slide: false,
            ..default()
        })
        .insert(Ccd::enabled())
        .insert(ColliderMassProperties::Mass(0.001))
        .insert(AnimationTimer(Timer::from_seconds(
            1. / idle.fps as f32,
            TimerMode::Repeating,
        )));
    commands.insert_resource(Animations {
        map: animations,
        active: idle,
    });
}

/* Read the character controller collisions stored in the character controller’s output. */
fn handle_player_collisions(
    wall_query: Query<&Wall>,
    plat_query: Query<&Velocity, (With<Platform>, Without<Player>)>,
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    mut character_controller_outputs: Query<
        (
            Entity,
            &mut Velocity,
            Option<&LastWall>,
            &KinematicCharacterControllerOutput,
        ),
        With<Player>,
    >,
) {
    for (player, mut player_velocity, last_wall, output) in character_controller_outputs.iter_mut()
    {
        for collision in &output.collisions {
            // Do something with that collision information.
            // info!("Collision: {:?}", collision); //TODO: move ReachedWall event here

            let collided_with = collision.entity;
            let new_wall = wall_query.get_component::<Wall>(collided_with).ok();
            match (last_wall, new_wall) {
                (Some(last), Some(new)) if last.0 != *new => {
                    info!("reached new wall");
                    commands.entity(player).insert(LastWall(*new));
                }
                (None, Some(new)) => {
                    info!("reached wall");
                    commands.entity(player).insert(LastWall(*new));
                }
                _ => {}
            }
            let Some(platform_velocity )= plat_query.get(collided_with).ok() else {
                continue;
            };
            let Some(contact) = rapier_context.contact_pair(player, collided_with) else {
                continue;
            };
            let Some(manifold) = contact.manifolds().next() else {
                continue;
            };
            if manifold.normal() == Vec2::new(0., -1.) {
                // commands
                //     .entity(player)
                //     .insert(OnPlatform(*platform_velocity));
                player_velocity.linvel.x = 0.;
                info!(
                    "player {:?} on platform {:?}",
                    player_velocity.linvel.x, platform_velocity.linvel.x
                );
            }
        }
    }
}

fn move_player(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform, &mut KinematicCharacterController), With<Player>>,
) {
    for (velocity, mut transform, mut controller) in query.iter_mut() {
        // velocity.linvel.x = velocity.linvel.x.clamp(-PLAYER_SPEED, PLAYER_SPEED);
        let translation = Vec3::new(
            velocity.linvel.x * time.delta_seconds(),
            velocity.linvel.y * time.delta_seconds(),
            0.,
        );
        // transform.translation += translation;
        controller.translation = Some(Vec2::new(translation.x, translation.y));
    }
}

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Facing), With<Player>>,
) {
    for (mut velocity, mut facing) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            velocity.linvel.x = -PLAYER_SPEED;
            *facing = Facing::Left;
        } else if keyboard_input.pressed(KeyCode::D) {
            velocity.linvel.x = PLAYER_SPEED;
            *facing = Facing::Right;
        }
        if keyboard_input.pressed(KeyCode::Space) && velocity.linvel.y.abs() <= 0.001 {
            velocity.linvel.y += JUMP_VELOCITY;
        }
    }
}

fn send_wall_reached_event(
    mut wall_reached_events: EventWriter<WallReached>,
    mut query: Query<(&LastWall, Changed<LastWall>), With<Player>>,
) {
    // only send the event if the wall that was reached is new
    if let Ok((wall, reached_new_wall)) = query.get_single_mut() {
        if reached_new_wall {
            wall_reached_events.send(WallReached(**wall));
        }
    };
}

fn change_player_animation(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(Entity, &Velocity), With<Player>>,
    mut animations: ResMut<Animations>,
) {
    for (player, velocity) in query.iter_mut() {
        let old_animation_handle = animations.active.handle.clone();
        animations.active = if velocity.linvel.y.abs() >= 0.1 {
            &animations.map["jump"]
        } else if velocity.linvel.x.abs() <= 0.001 {
            &animations.map["idle"]
        } else if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::D) {
            &animations.map["run"]
        } else {
            &animations.map["idle"]
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

fn despawn_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
