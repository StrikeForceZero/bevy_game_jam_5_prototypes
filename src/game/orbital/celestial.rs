use std::fmt::Formatter;

use avian2d::math::Vector;
use avian2d::prelude::{Collider, LinearVelocity, Mass, RigidBody};
use bevy::asset::Assets;
use bevy::color::palettes::basic::YELLOW;
use bevy::color::palettes::css::{RED, TEAL};
use bevy::ecs::query::QueryData;
use bevy::prelude::{
    Added, App, AppGizmoBuilder, Bundle, Circle, Color, Commands, Component, DetectChanges, Entity,
    FixedUpdate, GizmoConfigGroup, GizmoConfigStore, Gizmos, Handle, Mesh, Mut, Name, Query, Ref,
    Reflect, ReflectComponent, Res, ResMut, Startup, Transform, Update, Vec2, With,
};
use bevy::sprite::{ColorMaterial, ColorMesh2dBundle, Mesh2dHandle};
use bevy::ui::Display;
use bevy::utils::{default, EntityHash, EntityHashMap, HashMap};
use derive_more::Display;
use itertools::Itertools;
use log::debug;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::util::color_material_manager::ColorMaterialManager;
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId, PrototypeMeshManager};

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_systems(Update, on_added);
    app.add_systems(FixedUpdate, physics_update);
    app.init_gizmo_group::<DebugCelestialGizmos>()
        .add_systems(Update, draw_lines);
}

#[derive(Default, Reflect, GizmoConfigGroup, AutoRegisterType)]
pub struct DebugCelestialGizmos {
    point_map: EntityHashMap<Entity, Vec<Vec2>>,
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
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.mesh.transform = transform;
        self
    }

    pub fn with_static(mut self) -> Self {
        self.rigid_body = RigidBody::Static;
        self
    }

    pub fn with_dynamic(mut self) -> Self {
        self.rigid_body = RigidBody::Dynamic;
        self
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

#[derive(QueryData)]
#[query_data(mutable)]
struct PhysicsUpdateQueryData<'w> {
    entity: Entity,
    transform: &'w Transform,
    mass: &'w Mass,
    linear_velocity: Mut<'w, LinearVelocity>,
    collider: &'w Collider,
}

struct BodyPhysicsData {
    entity: Entity,
    mass: f32,
    pos: Vec2,
    velocity: Vec2,
}

impl From<PhysicsUpdateQueryDataReadOnlyItem<'_, '_>> for BodyPhysicsData {
    fn from(value: PhysicsUpdateQueryDataReadOnlyItem<'_, '_>) -> Self {
        Self {
            entity: value.entity,
            mass: value.mass.0,
            pos: value.transform.translation.truncate(),
            velocity: value.linear_velocity.0.into(),
        }
    }
}

fn compute_forces(bodies: &[BodyPhysicsData]) -> Vec<Vec2> {
    const G: f32 = 6.67430e-11; // gravitational constant in m^3 kg^-1 s^-2
    let mut forces = vec![Vec2::ZERO; bodies.len()];

    for (i, body_i) in bodies.iter().enumerate() {
        for (j, body_j) in bodies.iter().enumerate() {
            if i != j {
                let r_ij = body_j.pos - body_i.pos;
                let distance = r_ij.length();
                if distance > 0.0 {
                    let force_magnitude = G * body_i.mass * body_j.mass / (distance * distance);
                    let force_direction = r_ij.normalize();
                    let force = force_direction * force_magnitude;
                    forces[i] += force;
                }
            }
        }
    }

    forces
}

const FORCE_SCALE: f32 = 10000.0;
fn physics_update(
    mut celestial_q: Query<PhysicsUpdateQueryData, With<CelestialBody>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (_config, debug_gizmos) = config_store.config_mut::<DebugCelestialGizmos>();
    let bodies = celestial_q
        .iter()
        .map(BodyPhysicsData::from)
        .collect::<Vec<_>>();
    let forces = compute_forces(&bodies);
    let force_map = bodies
        .into_iter()
        .zip(forces.into_iter())
        .map(|(b, force)| (b.entity, force))
        .collect::<HashMap<_, _>>();
    for mut item in celestial_q.iter_mut() {
        let Some(force) = force_map.get(&item.entity) else {
            unreachable!();
        };
        let force = *force * FORCE_SCALE;
        // debug!("{} {force:?}", item.entity);
        item.linear_velocity.0 += Vector::from(force);
        let mut entry = debug_gizmos.point_map.entry(item.entity).or_default();

        if let Some(&last) = entry.last() {
            if last == item.transform.translation.truncate() {
                continue;
            }
        }
        entry.push(item.transform.translation.truncate());
    }
}

fn draw_lines(mut debug_gizmos: Gizmos<DebugCelestialGizmos>) {
    for (_, points) in debug_gizmos.config_ext.point_map.iter() {
        let mut color = Color::from(TEAL);
        for (a, b) in points.iter().rev().tuple_windows() {
            debug_gizmos.line_2d(*a, *b, color);
            let mut linear = color.to_linear();
            linear.alpha -= 0.0001;
            linear.alpha = linear.alpha.clamp(0.0, 1.0);
            color = Color::LinearRgba(linear);
            if linear.alpha == 0.0 {
                break;
            }
        }
    }
}
