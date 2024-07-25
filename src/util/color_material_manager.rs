use bevy::prelude::*;
use bevy::utils::HashMap;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::util::color_id::ColorId;
use crate::util::string::AnyString;

pub(super) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.init_resource::<ColorMaterialManager>();
}

pub type ColorMaterialManagerId = AnyString;

pub trait AssociatedColorMaterial {
    fn get_id(&self) -> ColorMaterialManagerId;
    fn get_color_material(&self) -> ColorMaterial;
}

impl AssociatedColorMaterial for Color {
    fn get_id(&self) -> ColorMaterialManagerId {
        format!("{:?}", ColorId::from(*self)).into()
    }

    fn get_color_material(&self) -> ColorMaterial {
        (*self).into()
    }
}

#[derive(Resource, Debug, Default, Clone, Reflect, AutoRegisterType)]
#[reflect(Resource)]
pub struct ColorMaterialManager {
    material_map: HashMap<ColorMaterialManagerId, Handle<ColorMaterial>>,
}

impl ColorMaterialManager {
    pub fn get_or_create<T: AssociatedColorMaterial>(
        &mut self,
        color_materials: &mut ResMut<Assets<ColorMaterial>>,
        color_or_trait_obj: T,
    ) -> Handle<ColorMaterial> {
        let id = color_or_trait_obj.get_id();
        let color_material = color_or_trait_obj.get_color_material();
        self.material_map
            .entry(id)
            .or_insert_with(|| color_materials.add(color_material))
            .clone()
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;
