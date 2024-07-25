use bevy::prelude::*;

use internal_proc_macros::RegisterTypeBinder;

pub mod platter;
pub mod platter_object;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_plugins(platter::plugin);
    app.add_plugins(platter_object::plugin);
}

#[derive(RegisterTypeBinder)]
pub struct Types;
