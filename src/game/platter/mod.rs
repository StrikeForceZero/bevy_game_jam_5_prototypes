use bevy::prelude::*;

use internal_proc_macros::RegisterTypeBinder;

pub mod mesh;
pub mod platter;
pub mod segment;
pub mod value;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_plugins(mesh::plugin);
    app.add_plugins(platter::plugin);
    app.add_plugins(segment::plugin);
    app.add_plugins(value::plugin);
}

#[derive(RegisterTypeBinder)]
pub struct Types;
