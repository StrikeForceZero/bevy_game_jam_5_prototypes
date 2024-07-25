use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};
use internal_shared::register_type_binder::RegisterTypeBinder;

use crate::util::color_material_manager::ColorMaterialManager;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_systems(Update, (outline_changed, outline_removed).chain());
}

#[derive(Component, Debug, SmartDefault, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct Outline {
    #[default(Color::WHITE)]
    color: Color,
    #[default(1.1)]
    size: f32,
}

#[derive(Component, Debug, SmartDefault, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct OutlineMeshMarker;

#[derive(RegisterTypeBinder)]
pub struct Types;

#[derive(QueryData)]
struct OutlineProcessQueryData<'w> {
    entity: Entity,
    transform: &'w Transform,
    outline: Ref<'w, Outline>,
    mesh_handle: Ref<'w, Mesh2dHandle>,
}

#[derive(SystemParam)]
struct OutlineProcessSystemParams<'w, 's> {
    commands: Commands<'w, 's>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    color_material_manager: ResMut<'w, ColorMaterialManager>,
    child_mesh_marker_q: Query<'w, 's, (), With<OutlineMeshMarker>>,
    child_q: Query<'w, 's, &'static Children>,
}

#[derive(SystemParam)]
struct OutlineQuerySystemParams<'w, 's> {
    outline_changed_q: Query<'w, 's, OutlineProcessQueryData<'static>, Changed<Outline>>,
    mesh_changed_q: Query<'w, 's, OutlineProcessQueryData<'static>, Changed<Handle<Mesh>>>,
}

fn process(
    process_params: &mut OutlineProcessSystemParams,
    query_item: OutlineProcessQueryDataItem,
) {
    debug!("process: {}", query_item.entity);
    let mut child = None;
    for child_entity in process_params.child_q.iter_descendants(query_item.entity) {
        if !process_params.child_mesh_marker_q.contains(child_entity) {
            continue;
        }
        if child.replace(child_entity).is_some() {
            panic!("more than one OutlineMeshMarker for {}", query_item.entity);
        }
    }
    let child = if let Some(child) = child {
        debug!("reusing old marker {child}");
        Some(child)
    } else {
        let Some(mut entity_commands) = process_params.commands.get_entity(query_item.entity)
        else {
            return;
        };
        debug!("creating new marker");
        let mut child = None;
        entity_commands.with_children(|parent| {
            child = Some(
                parent
                    .spawn((
                        Name::new("OutlineMeshMarker"),
                        OutlineMeshMarker,
                        SpatialBundle::default(),
                    ))
                    .id(),
            );
        });
        child
    };
    let Some(child) = child else {
        // probably clicking the OutlineMeshMarker in the inspector
        return;
    };
    let transform =
        Transform::from_xyz(0.0, 0.0, query_item.transform.translation.z - 1.0).with_scale(
            Vec3::new(query_item.outline.size, query_item.outline.size, 1.0),
        );
    let color_material_handle = process_params
        .color_material_manager
        .get_or_create(&mut process_params.materials, query_item.outline.color);
    process_params.commands.entity(child).try_insert((
        query_item.mesh_handle.clone(),
        color_material_handle,
        transform,
    ));
}

fn outline_changed(mut params: OutlineProcessSystemParams, query: OutlineQuerySystemParams) {
    fn should_process<T>(res: &Ref<T>) -> bool {
        res.is_added() || res.is_changed()
    }
    for item in query.outline_changed_q.iter() {
        debug!("process: {}", item.entity);
        if !should_process(&item.outline) {
            continue;
        }
        process(&mut params, item);
    }
    for item in query.mesh_changed_q.iter() {
        debug!("process: {}", item.entity);
        if !should_process(&item.mesh_handle) {
            continue;
        }
        process(&mut params, item);
    }
}

fn outline_removed(
    mut commands: Commands,
    mut removed_outline: RemovedComponents<Outline>,
    child_q: Query<&Children>,
    child_mesh_marker_q: Query<(), With<OutlineMeshMarker>>,
) {
    for removed in removed_outline.read() {
        debug!("removing outline for {removed}");
        for child in child_q.iter_descendants(removed) {
            if !child_mesh_marker_q.contains(child) {
                continue;
            }
            debug!("removing outline marker for {child}");
            let Some(entity_commands) = commands.get_entity(child) else {
                continue;
            };
            entity_commands.despawn_recursive();
        }
    }
}
