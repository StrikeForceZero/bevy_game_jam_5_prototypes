use bevy::asset::{Assets, Handle};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{App, ColorMaterial, Mesh, ResMut};
use bevy::sprite::ColorMesh2dBundle;
use bevy::utils::default;

use crate::game::util::debug_draw;
use crate::util::color_material_manager::{AssociatedColorMaterial, ColorMaterialManager};
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshManager};

pub(crate) mod color_ext;
pub(crate) mod color_id;
pub(crate) mod color_material_manager;
pub(crate) mod prototype_mesh_manager;
pub(crate) mod ref_ext;
pub(crate) mod string;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        string::plugin,
        debug_draw::plugin,
        color_id::plugin,
        color_material_manager::plugin,
        prototype_mesh_manager::plugin,
    ));
}

#[derive(SystemParam)]
pub struct PrototypeManagerSystemParam<'w> {
    pub materials: ResMut<'w, Assets<ColorMaterial>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub color_material_manager: ResMut<'w, ColorMaterialManager>,
    pub prototype_mesh_manager: ResMut<'w, PrototypeMeshManager>,
}

impl PrototypeManagerSystemParam<'_> {
    pub fn get_or_create_mesh<'a, T: PrototypeMesh + 'a>(
        &mut self,
        mesh: impl Into<&'a T>,
    ) -> Handle<Mesh> {
        self.prototype_mesh_manager
            .get_or_create(&mut self.meshes, mesh)
    }
    pub fn get_or_create_material<T: AssociatedColorMaterial>(
        &mut self,
        color: T,
    ) -> Handle<ColorMaterial> {
        self.color_material_manager
            .get_or_create(&mut self.materials, color)
    }

    pub fn get_or_create_color_mesh_2d<'a, M: PrototypeMesh + 'a, C: AssociatedColorMaterial>(
        &mut self,
        mesh: impl Into<&'a M>,
        color: C,
    ) -> ColorMesh2dBundle {
        ColorMesh2dBundle {
            mesh: self.get_or_create_mesh(mesh).into(),
            material: self.get_or_create_material(color),
            ..default()
        }
    }
}
