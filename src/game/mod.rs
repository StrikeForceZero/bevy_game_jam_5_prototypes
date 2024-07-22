//! Game mechanics and content.

use bevy::prelude::*;

mod animation;
pub mod assets;
pub mod audio;
mod camera;
mod movement;
mod orbital;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        camera::plugin,
        animation::plugin,
        audio::plugin,
        assets::plugin,
        movement::plugin,
        spawn::plugin,
        orbital::plugin,
    ));
}
