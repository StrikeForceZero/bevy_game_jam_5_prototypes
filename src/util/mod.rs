use bevy::asset::{Assets, Handle};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{App, Color, ColorMaterial, Mesh, ResMut};
use bevy::sprite::ColorMesh2dBundle;
use bevy::utils::default;

use crate::game::util::debug_draw;
use crate::util::color_material_manager::ColorMaterialManager;
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
    materials: ResMut<'w, Assets<ColorMaterial>>,
    meshes: ResMut<'w, Assets<Mesh>>,
    color_material_manager: ResMut<'w, ColorMaterialManager>,
    prototype_mesh_manager: ResMut<'w, PrototypeMeshManager>,
}

impl PrototypeManagerSystemParam<'_> {
    pub fn get_or_create_mesh<'a, T: PrototypeMesh + 'a>(
        &mut self,
        mesh: impl Into<&'a T>,
    ) -> Handle<Mesh> {
        self.prototype_mesh_manager
            .get_or_create(&mut self.meshes, mesh)
    }
    pub fn get_or_create_material(
        &mut self,
        color: impl Into<Color> + Copy,
    ) -> Handle<ColorMaterial> {
        self.color_material_manager
            .get_or_create(&mut self.materials, color)
    }

    pub fn get_or_create_color_mesh_2d<'a, T: PrototypeMesh + 'a>(
        &mut self,
        mesh: impl Into<&'a T>,
        color: impl Into<Color> + Copy,
    ) -> ColorMesh2dBundle {
        ColorMesh2dBundle {
            mesh: self.get_or_create_mesh(mesh).into(),
            material: self.get_or_create_material(color),
            ..default()
        }
    }
}
