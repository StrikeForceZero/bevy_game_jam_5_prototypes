use bevy::prelude::*;

pub mod sfx;
pub mod soundtrack;

pub fn plugin(app: &mut App) {
    app.add_plugins((sfx::plugin, soundtrack::plugin));
}
