use bevy::prelude::*;

pub mod debug_draw;
pub mod force;
pub mod mesh;
pub mod outline;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(outline::plugin);
}
