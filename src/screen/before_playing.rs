//! The screen state for the main game loop.

use bevy::prelude::*;

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::BeforePlaying), enter_before_playing);
}

fn enter_before_playing(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Playing);
}
