use bevy::prelude::App;

pub mod celestial;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        // multiline
        celestial::plugin,
    ));
}
