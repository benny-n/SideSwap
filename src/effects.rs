use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use rand::{random, seq::SliceRandom};

use crate::{events::WallReached, AppState};

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    Earthquake,
    FastPlatforms,
    InverseKeyboard,
    FallthroughPlatforms,
    HighGravity,
    LowGravity,
    Darkness,
    IcyPlatforms,
}

impl ToString for Effect {
    fn to_string(&self) -> String {
        format!("{:?}", self)
            .chars()
            .fold(String::new(), |mut acc, c| {
                if !acc.is_empty() && c.is_uppercase() {
                    acc.push(' ');
                }
                acc.push(c);
                acc
            })
    }
}

const EFFECTS: &[Effect] = &[
    Effect::Earthquake,
    Effect::FastPlatforms,
    Effect::FallthroughPlatforms,
    Effect::HighGravity,
    Effect::LowGravity,
    Effect::InverseKeyboard,
    Effect::Darkness,
    Effect::IcyPlatforms,
];

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct EffectQueue(pub Vec<Effect>);

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EffectQueue(vec![]))
            .add_systems((random_effect, play_sound_effect).chain())
            .add_systems(
                (
                    shake_camera,
                    change_gravity,
                    apply_darkness,
                    remove_darkness,
                )
                    .after(random_effect)
                    .in_set(OnUpdate(AppState::InGame)),
            );
    }
}

fn random_effect(mut effect_q: ResMut<EffectQueue>, mut event_reader: EventReader<WallReached>) {
    if event_reader.iter().next().is_some() {
        effect_q.pop();
        if effect_q.is_empty() {
            let mut effects = EFFECTS.to_vec();
            effects.shuffle(&mut rand::thread_rng());
            effect_q.0 = effects;
        }
    }
}

fn play_sound_effect(
    effect_q: Res<EffectQueue>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    if effect_q.is_changed() {
        let Some(effect) = effect_q.last() else {
            return;
        };
        let sound_effect = asset_server.load(format!(
            "sounds/{}.wav",
            effect.to_string().replace(' ', "")
        ));
        audio.play(sound_effect);
    }
}

fn shake_camera(
    effect_q: Res<EffectQueue>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    let Ok(mut ortho) = camera_query.get_single_mut() else {
        return;
    };
    if let Some(Effect::Earthquake) = effect_q.last() {
        let dz = 0.025 - random::<f32>() * 0.05;
        // Do not move the camera too much, clamp the value
        ortho.scale = f32::clamp(ortho.scale - dz, 0.975, 1.);
    } else {
        ortho.scale = 1.;
    }
}

fn change_gravity(effect_q: Res<EffectQueue>, mut gravity_query: Query<&mut GravityScale>) {
    if !effect_q.is_changed() {
        return;
    }
    let Ok(mut gravity) = gravity_query.get_single_mut() else {
        return;
    };
    gravity.0 = match effect_q.last() {
        Some(Effect::HighGravity) => 10.0,
        Some(Effect::LowGravity) => 2.5,
        _ => 5.0,
    }
}

#[derive(Component)]
pub struct Darkness;

fn apply_darkness(
    effect_q: Res<EffectQueue>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if !effect_q.is_changed() {
        return;
    }
    let Ok(window) = window_query.get_single() else {
        return;
    };
    if let Some(Effect::Darkness) = effect_q.last() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.0, 0.0, 0.0, 0.975),
                    custom_size: Some(Vec2::new(window.width() * 2., window.height() * 2.)),
                    ..default()
                },
                transform: Transform::from_xyz(window.height() / 2., window.width() / 2., 999.),
                ..default()
            },
            Darkness,
        ));
    }
}

fn remove_darkness(
    effect_q: Res<EffectQueue>,
    mut commands: Commands,
    darkness_query: Query<Entity, With<Darkness>>,
) {
    if !effect_q.is_changed() {
        return;
    }
    if let Some(Effect::Darkness) = effect_q.last() {
        return;
    }
    for entity in darkness_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
