use avian2d::prelude::{AngularVelocity, Collider, RigidBody};
use bevy::color::palettes::css::{GRAY, TEAL};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::sprite::Mesh2dHandle;
use derive_more::Display;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::platter::mesh::{
    PlatterMainMesh, PlatterMeshes, PlatterMeshOptions, PlatterMeshOptionsObj, PlatterSegmentMesh,
};
use crate::game::platter::segment::PlatterSegmentBundle;
use crate::game::util::mesh::{generate_donut_vertices, generate_subdivided_donut_split_vertices};
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId};
use crate::util::PrototypeManagerSystemParam;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct Platter;

#[derive(Bundle, SmartDefault, Clone)]
struct PlatterBundle {
    #[default(Name::new("Platter"))]
    name: Name,
    platter: Platter,
    platter_mesh_options: PlatterMeshOptions,
    platter_main_mesh: PlatterMainMesh,
    color_mesh2d_bundle: ColorMesh2dBundle,
    #[default(RigidBody::Kinematic)]
    rigid_body: RigidBody,
    angular_velocity: AngularVelocity,
}

impl PlatterBundle {
    fn new(
        prototype_context: &mut PrototypeManagerSystemParam,
        platter_mesh_options: PlatterMeshOptionsObj,
    ) -> (Self, Vec<PlatterSegmentMesh>) {
        let platter_meshes = PlatterMeshes::from(platter_mesh_options);
        if platter_meshes.main.vertices.is_empty() {
            panic!("empty vertices");
        }
        let color_mesh2d_bundle = prototype_context
            .get_or_create_color_mesh_2d(&platter_meshes.main, platter_mesh_options.main_color);
        let bundle = Self {
            platter_mesh_options: PlatterMeshOptions::new(platter_mesh_options),
            platter_main_mesh: platter_meshes.main,
            color_mesh2d_bundle,
            ..default()
        };
        (bundle, platter_meshes.segments)
    }
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.color_mesh2d_bundle.transform = transform;
        self
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct CreatePlatterOptions {
    pub platter_mesh_options: PlatterMeshOptionsObj,
    pub transform: Transform,
}

pub fn create_platter<'a>(
    mut entity_commands: EntityCommands<'a>,
    prototype_context: &mut PrototypeManagerSystemParam,
    options: CreatePlatterOptions,
) -> EntityCommands<'a> {
    let (platter_bundle, segment_meshes) =
        PlatterBundle::new(prototype_context, options.platter_mesh_options);
    let platter_bundle = platter_bundle.with_transform(options.transform);
    let segments = segment_meshes
        .into_iter()
        .map(|psm| PlatterSegmentBundle::new(prototype_context, psm));
    entity_commands
        .insert(platter_bundle)
        .with_children(move |parent| {
            for segment in segments {
                parent.spawn(segment);
            }
        });
    entity_commands
}

#[derive(RegisterTypeBinder)]
pub struct Types;
