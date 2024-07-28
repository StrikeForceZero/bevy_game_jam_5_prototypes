use bevy::color::Color;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::util::mesh::{
    generate_donut_vertices, generate_subdivided_donut_split_vertices, line_strip_2d_to_mesh,
};
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId};

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
}

#[derive(Component, Debug, SmartDefault, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterMeshOptions(PlatterMeshOptionsObj);

impl PlatterMeshOptions {
    pub fn new(options: PlatterMeshOptionsObj) -> Self {
        Self(options)
    }
    pub fn get(&self) -> &PlatterMeshOptionsObj {
        &self.0
    }
}

#[derive(Debug, SmartDefault, Copy, Clone, Reflect, AutoRegisterType)]
pub struct PlatterMeshOptionsObj {
    pub inner_radius: f32,
    pub outer_radius: f32,
    #[default(64)]
    pub outer_resolution: usize,
    #[default(32)]
    pub inner_resolution: usize,
    #[default(1)]
    pub pie_cuts: usize,
    #[default(1)]
    pub onion_layers: usize,
    #[default(Color::BLACK)]
    pub main_color: Color,
    pub initial_segment_color: Color,
}

#[derive(Component, Debug, Default, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub(super) struct PlatterMainMesh {
    pub(super) options: PlatterMeshOptionsObj,
    pub(super) vertices: Vec<Vec2>,
}

#[derive(Component, Debug, Default, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub(super) struct PlatterSegmentMesh {
    pub(super) options: PlatterMeshOptionsObj,
    pub(super) pie_cut: usize,
    pub(super) onion_layer: usize,
    pub(super) vertices: Vec<Vec2>,
}

pub(super) struct PlatterMeshes {
    pub(super) main: PlatterMainMesh,
    pub(super) segments: Vec<PlatterSegmentMesh>,
}

impl From<PlatterMeshOptionsObj> for PlatterMeshes {
    fn from(options: PlatterMeshOptionsObj) -> Self {
        let segments = generate_subdivided_donut_split_vertices(
            options.inner_radius,
            options.outer_radius,
            options.inner_resolution,
            options.outer_resolution,
            options.pie_cuts,
            options.onion_layers,
            true,
        );
        let segments = segments
            .into_iter()
            .enumerate()
            .flat_map(|(ix_pie_cut, pie_cut)| {
                pie_cut
                    .into_iter()
                    .enumerate()
                    .map(move |(ix_onion_layer, onion_layer)| PlatterSegmentMesh {
                        options,
                        pie_cut: ix_pie_cut,
                        onion_layer: ix_onion_layer,
                        vertices: onion_layer,
                    })
            })
            .collect();
        Self {
            main: PlatterMainMesh {
                options,
                vertices: generate_donut_vertices(
                    options.inner_radius,
                    options.outer_radius,
                    options.inner_resolution,
                    options.outer_resolution,
                    false,
                ),
            },
            segments,
        }
    }
}

impl PrototypeMesh for PlatterMainMesh {
    fn get_id(&self) -> PrototypeMeshId {
        format!("{self:?}").into()
    }

    fn get_mesh(&self) -> Mesh {
        line_strip_2d_to_mesh(self.vertices.clone())
    }
}

impl PrototypeMesh for PlatterSegmentMesh {
    fn get_id(&self) -> PrototypeMeshId {
        format!("{self:?}").into()
    }

    fn get_mesh(&self) -> Mesh {
        line_strip_2d_to_mesh(self.vertices.clone())
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;
