use bevy::prelude::*;

pub mod outline;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(outline::plugin);
}
