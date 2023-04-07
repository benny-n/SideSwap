use bevy::{prelude::*, utils::Instant};

use crate::{player::Player, AppState, Score, Wall};

pub const MAX_TIME_TO_REACH_WALL: f32 = 15.0;

#[derive(Resource, Clone, Copy)]
pub enum Reason {
    Died,
    OutOfTime,
}
pub struct WallReached(pub Wall);

pub struct Lost(pub Reason);
#[derive(Resource, Deref, DerefMut)]
pub struct GameTimer(pub Instant);
pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WallReached>()
            .add_event::<Lost>()
            .add_system(initialize_game_timer.in_schedule(OnEnter(AppState::InGame)))
            .add_systems(
                (
                    transition_to_restart,
                    update_score,
                    reset_game_timer,
                    is_dead,
                    is_out_of_time,
                )
                    .in_set(OnUpdate(AppState::InGame)),
            );
    }
}

fn initialize_game_timer(mut commands: Commands) {
    commands.insert_resource(GameTimer(Instant::now()));
}

fn reset_game_timer(mut timer: ResMut<GameTimer>, mut event: EventReader<WallReached>) {
    if event.iter().next().is_some() {
        timer.0 = Instant::now();
    }
}

fn transition_to_restart(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut event: EventReader<Lost>,
) {
    if let Some(reason) = event.into_iter().next() {
        state.set(AppState::YouDied);
        commands.insert_resource(reason.0)
    }
}

fn update_score(mut score: ResMut<Score>, mut event: EventReader<WallReached>) {
    if event.iter().next().is_some() {
        score.0 += 1;
    }
}

fn is_out_of_time(timer: Res<GameTimer>, mut event_writer: EventWriter<Lost>) {
    if timer.0.elapsed().as_secs_f32() > MAX_TIME_TO_REACH_WALL + 0.5 {
        event_writer.send(Lost(Reason::OutOfTime));
    }
}

fn is_dead(query: Query<&Transform, With<Player>>, mut event_writer: EventWriter<Lost>) {
    for transform in query.iter() {
        if transform.translation.y < -100. {
            event_writer.send(Lost(Reason::Died));
        }
    }
}
