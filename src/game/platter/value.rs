use bevy::color::palettes::css::*;
use bevy::prelude::*;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::platter::mesh::PlatterSegmentMesh;
use crate::util::color_material_manager::{AssociatedColorMaterial, ColorMaterialManagerId};
use crate::util::PrototypeManagerSystemParam;
use crate::util::ref_ext::RefExt;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_systems(Update, platter_value_updated);
}

#[derive(Debug, Copy, Clone, Reflect, AutoRegisterType)]
pub enum InnerValue {
    Red,
}

impl InnerValue {
    fn color(&self) -> Color {
        match self {
            InnerValue::Red => RED.into(),
        }
    }
}

impl AssociatedColorMaterial for InnerValue {
    fn get_id(&self) -> ColorMaterialManagerId {
        self.color().get_id()
    }

    fn get_color_material(&self) -> ColorMaterial {
        self.color().get_color_material()
    }
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterValue(pub Option<InnerValue>);

#[derive(RegisterTypeBinder)]
pub struct Types;

fn platter_value_updated(
    mut commands: Commands,
    mut prototype_manager_system_param: PrototypeManagerSystemParam,
    changed: Query<
        (Entity, Ref<PlatterValue>, &PlatterSegmentMesh),
        (Changed<PlatterValue>, With<Handle<ColorMaterial>>),
    >,
) {
    for (entity, value, psm) in changed.iter() {
        if !value.is_added_or_changed() {
            continue;
        }
        let color_material_handle = match value.0 {
            None => prototype_manager_system_param
                .get_or_create_material(psm.options.initial_segment_color),
            Some(inner) => prototype_manager_system_param.get_or_create_material(inner),
        };
        commands.entity(entity).insert(color_material_handle);
    }
}
