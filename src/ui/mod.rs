use bevy::prelude::*;

use crate::AppState;

use self::hud::{despawn_hud, spawn_hud, start_timer, update_timer};
use self::menu::{click_play_button, despawn_main_menu, spawn_main_menu, ButtonColors};

mod hud;
mod menu;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .add_system(spawn_main_menu.in_schedule(OnEnter(AppState::MainMenu)))
            .add_system(click_play_button.in_set(OnUpdate(AppState::MainMenu)))
            .add_system(despawn_main_menu.in_schedule(OnExit(AppState::MainMenu)))
            .add_systems((spawn_hud, start_timer).in_schedule(OnEnter(AppState::InGame)))
            .add_system(update_timer.in_set(OnUpdate(AppState::InGame)))
            .add_system(despawn_hud.in_schedule(OnExit(AppState::InGame)));
    }
}
