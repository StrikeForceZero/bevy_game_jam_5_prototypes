use bevy::color::palettes::css::*;
use bevy::prelude::*;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::platter::mesh::PlatterSegmentMesh;
use crate::game::platter::segment::PlatterSegmentColor;
use crate::util::color_material_manager::{AssociatedColorMaterial, ColorMaterialManagerId};
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
pub struct PlatterSegmentValue(pub Option<InnerValue>);

#[derive(RegisterTypeBinder)]
pub struct Types;

fn platter_value_updated(
    mut changed: Query<
        (
            Entity,
            Ref<PlatterSegmentValue>,
            &PlatterSegmentMesh,
            Mut<PlatterSegmentColor>,
        ),
        (Changed<PlatterSegmentValue>, With<PlatterSegmentColor>),
    >,
) {
    for (entity, value, psm, mut psc) in changed.iter_mut() {
        if !value.is_added_or_changed() {
            continue;
        }
        let new_color = match value.0 {
            None => psm.options.initial_segment_color,
            Some(inner) => inner.color(),
        };
        psc.0 = new_color;
    }
}
