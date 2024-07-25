use avian2d::prelude::{AngularVelocity, Collider, RigidBody};
use bevy::color::palettes::css::{GRAY, TEAL};
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use derive_more::Display;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::util::mesh::generate_donut_vertices;
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId};
use crate::util::PrototypeManagerSystemParam;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct Platter;

#[derive(Component, Debug, Default, Display, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterRadius(f32);

impl PrototypeMesh for PlatterRadius {
    fn get_id(&self) -> PrototypeMeshId {
        format!("{self:?}").into()
    }

    fn get_mesh(&self) -> Mesh {
        Circle::new(self.0).into()
    }
}

#[derive(Bundle, SmartDefault, Clone)]
pub struct PlatterBundle {
    #[default(Name::new("Platter"))]
    name: Name,
    platter: Platter,
    platter_radius: PlatterRadius,
    color_mesh_2d_bundle: ColorMesh2dBundle,
    collider: Collider,
    #[default(RigidBody::Kinematic)]
    rigid_body: RigidBody,
    angular_velocity: AngularVelocity,
}

impl PlatterBundle {
    pub fn new(
        prototype_manager_system_param: &mut PrototypeManagerSystemParam,
        radius: f32,
    ) -> Self {
        let platter_radius = PlatterRadius(radius);
        let color_mesh_2d_bundle =
            prototype_manager_system_param.get_or_create_color_mesh_2d(&platter_radius, GRAY);
        let collider = Collider::polyline(
            generate_donut_vertices(radius, radius * 3.0, 64),
            None,
        );
        Self {
            color_mesh_2d_bundle,
            collider,
            ..default()
        }
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;
