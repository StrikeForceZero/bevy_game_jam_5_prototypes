use bevy::prelude::*;
use bevy::utils::HashMap;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::util::color_id::ColorId;

pub(super) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.init_resource::<ColorMaterialManager>();
}

#[derive(Resource, Debug, Default, Clone, Reflect, AutoRegisterType)]
#[reflect(Resource)]
pub struct ColorMaterialManager {
    material_map: HashMap<ColorId, Handle<ColorMaterial>>,
}

impl ColorMaterialManager {
    pub fn get_or_create<T: Into<Color> + Copy>(
        &mut self,
        color_materials: &mut ResMut<Assets<ColorMaterial>>,
        color: T,
    ) -> Handle<ColorMaterial> {
        let color = color.into();
        self.material_map
            .entry(ColorId::from(color))
            .or_insert_with(|| color_materials.add(color))
            .clone()
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;
