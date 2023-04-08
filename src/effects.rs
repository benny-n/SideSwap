use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::GravityScale;
use rand::{random, seq::SliceRandom};

use crate::{events::WallReached, AppState};

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    Earthquake,
    // FastPlatforms,
    InverseKeyboard,
    FallthroughPlatforms,
    HighGravity,
}

impl ToString for Effect {
    fn to_string(&self) -> String {
        match self {
            Effect::Earthquake => "Earthquake",
            Effect::FallthroughPlatforms => "Fallthrough Platforms",
            Effect::HighGravity => "High Gravity",
            Effect::InverseKeyboard => "Inverse Keyboard",
        }
        .into()
    }
}

const EFFECTS: [Effect; 4] = [
    Effect::Earthquake,
    Effect::FallthroughPlatforms,
    Effect::HighGravity,
    Effect::InverseKeyboard,
];

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct EffectQueue(pub Vec<Effect>);

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EffectQueue(vec![])).add_systems(
            (
                random_effect.before(shake_camera).before(high_gravity),
                shake_camera,
                high_gravity,
            )
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
        // info!("Effect: {:?}", effect_q);
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

fn high_gravity(effect_q: Res<EffectQueue>, mut gravity_query: Query<&mut GravityScale>) {
    if !effect_q.is_changed() {
        return;
    }
    let Ok(mut gravity) = gravity_query.get_single_mut() else {
        return;
    };
    if let Some(Effect::HighGravity) = effect_q.last() {
        gravity.0 = 10.0;
    } else {
        gravity.0 = 5.0
    }
}
