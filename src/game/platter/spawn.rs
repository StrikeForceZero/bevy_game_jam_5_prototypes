use avian2d::collision::Collider;
use avian2d::prelude::{ColliderConstructor, ColliderMarker};
use bevy::prelude::*;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};
use internal_shared::register_type_binder::RegisterTypeBinder;

use crate::game::platter::mesh::PlatterMeshOptionsObj;
use crate::game::util::mesh::{generate_donut_vertices, generate_donut_vertices_clamped};
use crate::util::PrototypeManagerSystemParam;

pub(crate) fn plugin(app: &mut App) {
    crate::game::platter::arm::Types.register_types(app);
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct SpawnArea;

#[derive(Bundle, SmartDefault, Clone)]
pub struct SpawnAreaBundle {
    #[default("SpawnArea")]
    name: Name,
    spawn_area: SpawnArea,
    spatial_bundle: SpatialBundle,
    #[default(ColliderConstructor::Circle { radius: 0.0 })]
    collider_constructor: ColliderConstructor,
}

impl SpawnAreaBundle {
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.spatial_bundle.transform = transform;
        self
    }
    pub fn new(options: PlatterMeshOptionsObj) -> Self {
        let points = generate_donut_vertices_clamped(
            options.inner_radius,
            options.outer_radius,
            options.inner_resolution,
            options.outer_resolution,
            30f32.to_radians(),
            150f32.to_radians(),
            true,
        );
        Self {
            collider_constructor: ColliderConstructor::ConvexHull { points },
            ..default()
        }
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;
