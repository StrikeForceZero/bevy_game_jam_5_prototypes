use bevy::prelude::*;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};
use internal_shared::register_type_binder::RegisterTypeBinder;

use crate::util::PrototypeManagerSystemParam;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterArm;

#[derive(RegisterTypeBinder)]
pub struct Types;
