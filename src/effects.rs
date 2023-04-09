use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::GravityScale;
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
}

impl ToString for Effect {
    fn to_string(&self) -> String {
        match self {
            Effect::Earthquake => "Earthquake",
            Effect::FastPlatforms => "Fast Platforms",
            Effect::FallthroughPlatforms => "Fallthrough Platforms",
            Effect::HighGravity => "High Gravity",
            Effect::LowGravity => "Low Gravity",
            Effect::InverseKeyboard => "Inverse Keyboard",
        }
        .into()
    }
}

const EFFECTS: [Effect; 6] = [
    Effect::Earthquake,
    Effect::FastPlatforms,
    Effect::FallthroughPlatforms,
    Effect::HighGravity,
    Effect::LowGravity,
    Effect::InverseKeyboard,
];

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct EffectQueue(pub Vec<Effect>);

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EffectQueue(vec![]))
            .add_systems((random_effect, play_sound_effect).chain())
            .add_systems(
                (shake_camera, change_gravity)
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
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let (Ok(window), Ok(mut transform)) = (window_query.get_single(), camera_query.get_single_mut()) else {
        return;
    };
    if let Some(Effect::Earthquake) = effect_q.last() {
        let dx = 2.5 - random::<f32>() * 5.0;
        let dy = 7.5 - random::<f32>() * 15.0;
        // Do not move the camera too much, clamp the values
        let x = window.width() / 2.;
        let y = window.height() / 2.;
        transform.translation.x = f32::clamp(transform.translation.x + dx, x - 3., x + 3.);
        transform.translation.y = f32::clamp(transform.translation.y + dy, y - 20., y + 20.);
    } else {
        transform.translation.x = window.width() / 2.;
        transform.translation.y = window.height() / 2.;
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
