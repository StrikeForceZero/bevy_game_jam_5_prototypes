use std::hash::{Hash, Hasher};

use bevy::app::App;
use bevy::prelude::Reflect;
use derive_more::Display;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

pub(super) fn plugin(app: &mut App) {
    Types.register_types(app);
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Display, Reflect, AutoRegisterType)]
pub enum AnyString {
    Owned(String),
    Static(&'static str),
}

impl From<String> for AnyString {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}

impl From<&String> for AnyString {
    fn from(value: &String) -> Self {
        Self::Owned(value.to_owned())
    }
}

impl From<&'static str> for AnyString {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

#[derive(Debug, Clone, Display, Reflect, AutoRegisterType)]
pub enum AnyUniqueString {
    #[display(fmt = "{}", _0)]
    Owned(String),
    #[display(fmt = "{}", _0)]
    Static(&'static str),
}

impl From<String> for AnyUniqueString {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}

impl From<&String> for AnyUniqueString {
    fn from(value: &String) -> Self {
        Self::Owned(value.to_owned())
    }
}

impl From<&'static str> for AnyUniqueString {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

impl Hash for AnyUniqueString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl PartialEq for AnyUniqueString {
    fn eq(&self, other: &Self) -> bool {
        self.to_string().eq(&other.to_string())
    }
}

impl Eq for AnyUniqueString {}

#[derive(RegisterTypeBinder)]
pub struct Types;
