use std::hash::{Hash, Hasher};

use bevy::prelude::*;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

pub(super) fn plugin(app: &mut App) {
    Types.register_types(app);
}

#[derive(Debug, Copy, Clone, Reflect, AutoRegisterType)]
pub struct ColorId {
    color: Color,
}

impl PartialEq for ColorId {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for ColorId {}

impl ColorId {
    pub fn id(&self) -> [usize; 4] {
        let color = self.color.to_linear();
        let r = (color.red * 100.0) as usize;
        let g = (color.green * 100.0) as usize;
        let b = (color.blue * 100.0) as usize;
        let a = (color.alpha * 100.0) as usize;
        [r, g, b, a]
    }
}

impl Hash for ColorId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl From<Color> for ColorId {
    fn from(value: Color) -> Self {
        Self { color: value }
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;
