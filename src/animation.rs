use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::player::Facing;

#[derive(Component, Clone)]
pub struct Animation {
    pub handle: Handle<TextureAtlas>,
    pub curr: usize,
    pub last: usize,
    pub fps: usize,
}

#[derive(Resource)]
pub struct Animations {
    pub map: HashMap<String, Animation>,
    pub active: Animation,
}

pub struct AnimatorPlugin;

impl Plugin for AnimatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprites);
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

pub fn animate_sprites(
    time: Res<Time>,
    animations: Res<Animations>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite, &Facing)>,
) {
    for (mut timer, mut sprite, facing) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = (sprite.index + 1) % animations.active.last;
        }
        match facing {
            Facing::Left => sprite.flip_x = true,
            Facing::Right => sprite.flip_x = false,
        }
    }
}
