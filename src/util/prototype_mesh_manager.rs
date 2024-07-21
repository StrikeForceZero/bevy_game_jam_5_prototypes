use bevy::app::App;
use bevy::prelude::{Assets, Handle, Mesh, Reflect, ReflectResource, ResMut, Resource};
use bevy::utils::HashMap;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

pub(super) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.init_resource::<PrototypeMeshManager>();
}

pub trait PrototypeMesh {
    fn get_id(&self) -> &'static str;
    fn get_mesh(&self) -> Mesh;
}

#[derive(Resource, Debug, Default, Clone, Reflect, AutoRegisterType)]
#[reflect(Resource)]
pub struct PrototypeMeshManager {
    mesh_map: HashMap<&'static str, Handle<Mesh>>,
}

impl PrototypeMeshManager {
    pub fn get_or_create<T: PrototypeMesh>(
        &mut self,
        meshes: &mut ResMut<Assets<Mesh>>,
        prototype: T,
    ) -> Handle<Mesh> {
        self.mesh_map
            .entry(prototype.get_id())
            .or_insert_with(|| meshes.add(prototype.get_mesh()))
            .clone()
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;
