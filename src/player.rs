use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{Animation, AnimationTimer, Animations},
    effects::{Effect, EffectQueue},
    events::WallReached,
    tiles::Platform,
    AppState, Wall,
};

const PLAYER_SPEED: f32 = 125.;
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
        .insert(Facing::default())
        .insert(SpriteSheetBundle {
            texture_atlas: idle.handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(10., HALF_PLAYER_SIZE + 250., 0.),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
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
            snap_to_ground: Some(CharacterLength::Relative(0.5)),
            slide: false,
            ..default()
        })
        .insert(Ccd::enabled())
        .insert(ActiveEvents::COLLISION_EVENTS)
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
    plat_query: Query<&Transform, (With<Platform>, Without<Player>)>,
    mut commands: Commands,
    mut character_controller_outputs: Query<
        (
            Entity,
            &Transform,
            Option<&LastWall>,
            Option<&ImpulseJoint>,
            &KinematicCharacterControllerOutput,
        ),
        With<Player>,
    >,
) {
    for (player, player_transform, last_wall, joint, output) in
        character_controller_outputs.iter_mut()
    {
        for collision in &output.collisions {
            let collided_with = collision.entity;
            let new_wall = wall_query.get_component::<Wall>(collided_with).ok();
            if let Some(new) = new_wall {
                if last_wall.is_none() || matches!(last_wall, Some(last) if last.0 != *new) {
                    commands
                        .entity(player)
                        .insert(LastWall(*new))
                        .remove::<ImpulseJoint>();
                }
            }
            let Some(platform_transform )= plat_query.get(collided_with).ok() else {
                continue;
            };
            if joint.is_none() {
                let joint = FixedJointBuilder::new().local_anchor2(Vec2::new(
                    (platform_transform.translation - player_transform.translation).x,
                    -HALF_PLAYER_SIZE,
                ));
                commands
                    .entity(player)
                    .insert(ImpulseJoint::new(collided_with, joint));
            }
        }
    }
}

fn move_player(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut KinematicCharacterController), With<Player>>,
) {
    for (velocity, mut controller) in query.iter_mut() {
        controller.translation = Some(velocity.linvel * time.delta_seconds());
    }
}

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    effects_q: Res<EffectQueue>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Velocity, &mut Facing), With<Player>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    for (player, transform, mut velocity, mut facing) in query.iter_mut() {
        // TODO: This is a hack to prevent the player from falling off the screen.
        // It should be replaced with a proper solution.
        if transform.translation.y >= window.height() / 2. {
            continue;
        }
        let (left, right) = match effects_q.last() {
            Some(Effect::InverseKeyboard) => (KeyCode::D, KeyCode::A),
            _ => (KeyCode::A, KeyCode::D),
        };
        if keyboard_input.pressed(left) {
            velocity.linvel.x = -PLAYER_SPEED;
            *facing = Facing::Left;
            commands.entity(player).remove::<ImpulseJoint>();
        } else if keyboard_input.pressed(right) {
            velocity.linvel.x = PLAYER_SPEED;
            *facing = Facing::Right;
            commands.entity(player).remove::<ImpulseJoint>();
        }
        if keyboard_input.pressed(KeyCode::Space) && velocity.linvel.y.abs() <= 0.001 {
            velocity.linvel.y += JUMP_VELOCITY;
            velocity.linvel.x = 0.;
            commands.entity(player).remove::<ImpulseJoint>();
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
        animations.active = if velocity.linvel.y.abs() >= 0.01 {
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
