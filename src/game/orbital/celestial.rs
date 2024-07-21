use std::fmt::Formatter;

use avian2d::prelude::{Collider, Mass, RigidBody};
use bevy::asset::Assets;
use bevy::color::palettes::css::RED;
use bevy::prelude::{
    Added, App, Bundle, Circle, Color, Commands, Component, debug, DetectChanges, Entity, Handle,
    Mesh, Name, Query, Ref, Reflect, ReflectComponent, ResMut, Startup, Update,
};
use bevy::sprite::{ColorMaterial, ColorMesh2dBundle, Mesh2dHandle};
use bevy::ui::Display;
use bevy::utils::default;
use derive_more::Display;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::util::color_material_manager::ColorMaterialManager;
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId, PrototypeMeshManager};

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_systems(Update, on_added);
}

#[derive(Component, Debug, Copy, Clone, Hash, Eq, PartialEq, Display, SmartDefault)]
pub enum CelestialMesh {
    #[default]
    Standard(ordered_float::OrderedFloat<f32>),
}

impl PrototypeMesh for CelestialMesh {
    fn get_id(&self) -> PrototypeMeshId {
        self.to_string().into()
    }

    fn get_mesh(&self) -> Mesh {
        match *self {
            CelestialMesh::Standard(radius) => Circle::new(*radius).into(),
        }
    }
}

#[derive(Component, Debug, Clone, Default, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct CelestialBody;

#[derive(Component, Debug, Clone, Default, Reflect, AutoRegisterType)]
pub struct CelestialBodyColor(pub Color);

#[derive(Bundle, Clone, Default)]
pub struct CelestialBodyBundle {
    celestial_body: CelestialBody,
    rigid_body: RigidBody,
    collider: Collider,
    mass: Mass,
    mesh: ColorMesh2dBundle,
    celestial_mesh: CelestialMesh,
    celestial_body_color: CelestialBodyColor,
}

impl CelestialBodyBundle {
    pub fn standard(radius: f32, mass: f32, color: impl Into<Color>) -> Self {
        Self {
            celestial_body: CelestialBody,
            rigid_body: RigidBody::Kinematic,
            collider: Collider::circle(radius),
            mass: Mass(mass),
            mesh: ColorMesh2dBundle::default(),
            celestial_mesh: CelestialMesh::Standard(ordered_float::OrderedFloat(radius)),
            celestial_body_color: CelestialBodyColor(color.into()),
        }
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;

fn on_added(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut prototype_mesh_manager: ResMut<PrototypeMeshManager>,
    mut color_material_manager: ResMut<ColorMaterialManager>,
    added_color_q: Query<(Entity, Ref<CelestialBodyColor>), Added<CelestialBodyColor>>,
    added_mesh_q: Query<(Entity, Ref<CelestialMesh>), Added<CelestialMesh>>,
) {
    for (entity, color) in added_color_q.iter() {
        if !color.is_added() {
            continue;
        }
        let color = color.0;
        debug!("{entity} added color {color:?}");
        commands
            .entity(entity)
            .remove::<CelestialBodyColor>()
            .insert(color_material_manager.get_or_create(&mut materials, color));
    }
    for (entity, celestial_mesh) in added_mesh_q.iter() {
        if !celestial_mesh.is_added() {
            continue;
        }
        debug!("{entity} added mesh {celestial_mesh:?}");
        commands
            .entity(entity)
            .remove::<CelestialMesh>()
            .insert(Mesh2dHandle(
                prototype_mesh_manager
                    .get_or_create::<CelestialMesh>(&mut meshes, celestial_mesh.into_inner()),
            ));
    }
}
