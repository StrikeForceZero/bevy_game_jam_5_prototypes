//! Game mechanics and content.

use bevy::prelude::*;

mod animation;
pub mod assets;
pub mod audio;
pub mod camera;
mod movement;
pub mod orbital;
pub mod spawn;
pub mod util;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        util::plugin,
        camera::plugin,
        animation::plugin,
        audio::plugin,
        assets::plugin,
        movement::plugin,
        spawn::plugin,
        orbital::plugin,
    ));
}
