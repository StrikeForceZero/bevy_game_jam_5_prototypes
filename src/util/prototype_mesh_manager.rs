use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use bevy::app::App;
use bevy::prelude::{Assets, Handle, Mesh, Reflect, ReflectResource, ResMut, Resource};
use bevy::utils::HashMap;
use derive_more::Display;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

pub(super) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.init_resource::<PrototypeMeshManager>();
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Display, Reflect, AutoRegisterType)]
pub enum PrototypeMeshId {
    Owned(String),
    Static(&'static str),
}

impl From<String> for PrototypeMeshId {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}

impl From<&String> for PrototypeMeshId {
    fn from(value: &String) -> Self {
        Self::Owned(value.to_owned())
    }
}

impl From<&'static str> for PrototypeMeshId {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

pub trait PrototypeMesh {
    fn get_id(&self) -> PrototypeMeshId;
    fn get_mesh(&self) -> Mesh;
}

#[derive(Resource, Debug, Default, Clone, Reflect, AutoRegisterType)]
#[reflect(Resource)]
pub struct PrototypeMeshManager {
    mesh_map: HashMap<PrototypeMeshId, Handle<Mesh>>,
}

impl PrototypeMeshManager {
    pub fn get_or_create<'a, T: PrototypeMesh + 'a>(
        &mut self,
        meshes: &mut ResMut<Assets<Mesh>>,
        prototype: impl Into<&'a T>,
    ) -> Handle<Mesh> {
        let prototype = prototype.into();
        self.mesh_map
            .entry(prototype.get_id())
            .or_insert_with(|| meshes.add(prototype.get_mesh()))
            .clone()
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;
