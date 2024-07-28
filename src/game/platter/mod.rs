use bevy::prelude::*;

use internal_proc_macros::RegisterTypeBinder;

pub mod arm;
pub mod falling;
pub mod mesh;
pub mod platter;
pub mod segment;
pub mod spawn;
pub mod value;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_plugins(mesh::plugin);
    app.add_plugins(platter::plugin);
    app.add_plugins(segment::plugin);
    app.add_plugins(value::plugin);
    app.add_plugins(spawn::plugin);
    app.add_plugins(falling::plugin);
}

#[derive(RegisterTypeBinder)]
pub struct Types;
