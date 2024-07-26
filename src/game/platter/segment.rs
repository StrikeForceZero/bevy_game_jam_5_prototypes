use avian2d::prelude::Collider;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::transform::systems::propagate_transforms;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::platter::mesh::PlatterSegmentMesh;
use crate::game::platter::platter::Platter;
use crate::game::platter::value::PlatterSegmentValue;
use crate::game::util::mesh::calculate_centroid;
use crate::util::PrototypeManagerSystemParam;
use crate::util::ref_ext::RefExt;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_systems(PostUpdate, update_center_point.after(propagate_transforms));
    app.add_systems(Update, update_color);
    app.add_systems(Update, fix_z_index);
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterSegment;

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterSegmentColor(pub(super) Color);

impl PlatterSegmentColor {
    pub fn get(&self) -> Color {
        self.0
    }
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct CenterPoint(Vec2);

impl CenterPoint {
    pub fn get(&self) -> Vec2 {
        self.0
    }
}

#[derive(Bundle, SmartDefault, Clone)]
pub struct PlatterSegmentBundle {
    #[default(Name::new("PlatterSegment"))]
    name: Name,
    platter_segment: PlatterSegment,
    platter_segment_color: PlatterSegmentColor,
    platter_segment_mesh: PlatterSegmentMesh,
    color_mesh_2d_bundle: ColorMesh2dBundle,
    center_point: CenterPoint,
    collider: Collider,
    platter_segment_value: PlatterSegmentValue,
}

impl PlatterSegmentBundle {
    pub fn new(
        prototype_context: &mut PrototypeManagerSystemParam,
        platter_segment_mesh: PlatterSegmentMesh,
    ) -> Self {
        let pie_cut = platter_segment_mesh.pie_cut;
        let onion_layer = platter_segment_mesh.onion_layer;
        let color = platter_segment_mesh.options.initial_segment_color;
        if platter_segment_mesh.vertices.is_empty() {
            panic!("empty vertices");
        }
        let collider = Collider::polyline(platter_segment_mesh.vertices.clone(), None);
        let color_mesh_2d_bundle =
            prototype_context.get_or_create_color_mesh_2d(&platter_segment_mesh, color);
        Self {
            name: Name::new(format!("PlatterSegment({pie_cut},{onion_layer})")),
            platter_segment_color: PlatterSegmentColor(color),
            platter_segment_mesh,
            color_mesh_2d_bundle,
            collider,
            ..default()
        }
    }
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.color_mesh_2d_bundle.transform = transform;
        self
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;

#[derive(QueryData)]
struct PlatterQuery<'w> {
    children: &'w Children,
    global_transform: Ref<'w, GlobalTransform>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct PlatterSegmentQuery<'w> {
    entity: Entity,
    parent: &'w Parent,
    mesh: &'w PlatterSegmentMesh,
    center_point: Mut<'w, CenterPoint>,
    global_transform: Ref<'w, GlobalTransform>,
}

#[derive(SystemParam)]
struct UpdateCenterPointSystemParam<'w, 's> {
    platter_q: Query<'w, 's, PlatterQuery<'static>, (With<Platter>, Changed<GlobalTransform>)>,
    platter_seg_q: Query<'w, 's, PlatterSegmentQuery<'static>, With<PlatterSegment>>,
}

fn update_center_point(mut params: UpdateCenterPointSystemParam) {
    let mut children_to_check = vec![];
    for platter in params.platter_q.iter() {
        if !platter.global_transform.is_added_or_changed() {
            continue;
        }
        for child in platter.children {
            children_to_check.push(child);
        }
    }
    if children_to_check.is_empty() {
        return;
    }
    let platter_segments_to_update = children_to_check
        .into_iter()
        .filter_map(|&child| params.platter_seg_q.get(child).ok().map(|item| item.entity))
        .collect::<Vec<_>>();
    for platter_segment_entity in platter_segments_to_update {
        let Some(mut platter_segment) = params.platter_seg_q.get_mut(platter_segment_entity).ok()
        else {
            continue;
        };
        if !platter_segment.global_transform.is_added_or_changed() {
            continue;
        }
        if platter_segment.mesh.vertices.is_empty() {
            log::warn!("vertices empty for {}", platter_segment.entity);
            continue;
        }
        let Some(platter) = params.platter_q.get(platter_segment.parent.get()).ok() else {
            continue;
        };
        let new_center_point = platter
            .global_transform
            .mul_transform(platter_segment.global_transform.compute_transform())
            .transform_point(calculate_centroid(&platter_segment.mesh.vertices).extend(0.0))
            .truncate();
        if new_center_point == platter_segment.center_point.0 {
            continue;
        }
        platter_segment.center_point.0 = new_center_point;
    }
}

fn update_color(
    mut commands: Commands,
    mut prototype_manager_system_param: PrototypeManagerSystemParam,
    segments_q: Query<
        (Entity, Ref<PlatterSegmentColor>),
        (With<PlatterSegment>, Changed<PlatterSegmentColor>),
    >,
) {
    for (entity, color) in segments_q.iter() {
        if !color.is_added_or_changed() {
            continue;
        }
        commands
            .entity(entity)
            .insert(prototype_manager_system_param.get_or_create_material(color.0));
    }
}

fn fix_z_index(
    mut added_segments_q: Query<
        (Ref<PlatterSegment>, &Parent, &mut Transform),
        (Added<PlatterSegment>, Without<Platter>),
    >,
    platter_q: Query<&Transform, With<Platter>>,
) {
    for (platter_segment, parent, mut transform) in added_segments_q.iter_mut() {
        if !platter_segment.is_added_or_changed() {
            continue;
        }
        let Some(platter_transform) = platter_q.get(parent.get()).ok() else {
            panic!("orphaned platter segment");
        };
        transform.translation.z = platter_transform.translation.z + 1.0;
    }
}
