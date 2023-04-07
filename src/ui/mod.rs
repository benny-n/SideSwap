use bevy::prelude::*;

use crate::AppState;

use self::hud::{despawn_hud, spawn_hud, start_timer, update_score, update_timer};
use self::menu::{despawn_main_menu, spawn_main_menu};
use self::replay::{despawn_replay_screen, spawn_replay_screen};

mod hud;
mod menu;
mod replay;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .add_system(spawn_main_menu.in_schedule(OnEnter(AppState::MainMenu)))
            .add_system(spawn_replay_screen.in_schedule(OnEnter(AppState::YouDied)))
            .add_systems((spawn_hud, start_timer).in_schedule(OnEnter(AppState::InGame)))
            .add_system(click_play_button.in_set(OnUpdate(AppState::MainMenu)))
            .add_system(click_play_button.in_set(OnUpdate(AppState::YouDied)))
            .add_systems((update_timer, update_score).in_set(OnUpdate(AppState::InGame)))
            .add_system(despawn_main_menu.in_schedule(OnExit(AppState::MainMenu)))
            .add_system(despawn_hud.in_schedule(OnExit(AppState::InGame)))
            .add_system(despawn_replay_screen.in_schedule(OnExit(AppState::YouDied)));
    }
}

#[derive(Resource)]
pub struct ButtonColors {
    default: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            default: Color::CRIMSON,
            hovered: {
                let [r, g, b, _] = Color::CRIMSON.as_rgba_f32();
                Color::rgba(r, g, b, 0.5)
            },
        }
    }
}

pub fn click_play_button(
    button_colors: Res<ButtonColors>,
    mut state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.default.into();
            }
        }
    }
}
