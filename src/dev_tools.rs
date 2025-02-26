//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};

use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(crate::ui::inspector::plugin);
    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);
}
