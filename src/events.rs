use bevy::prelude::*;

use crate::{AppState, Score, Wall};

pub struct WallReached(pub Wall);
pub struct Died;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WallReached>()
            .add_event::<Died>()
            .add_systems((transition_to_restart, update_score).in_set(OnUpdate(AppState::InGame)));
    }
}

fn transition_to_restart(mut state: ResMut<NextState<AppState>>, mut event: EventReader<Died>) {
    if event.iter().next().is_some() {
        state.set(AppState::YouDied);
    }
}

fn update_score(mut score: ResMut<Score>, mut event: EventReader<WallReached>) {
    if event.iter().next().is_some() {
        score.0 += 1;
    }
}
