use std::fmt::Formatter;

use avian2d::math::Vector;
use avian2d::prelude::{Collider, LinearVelocity, Mass, RigidBody};
use bevy::asset::Assets;
use bevy::color::palettes::basic::YELLOW;
use bevy::color::palettes::css::{RED, TEAL};
use bevy::ecs::query::QueryData;
use bevy::prelude::{
    Added, App, AppGizmoBuilder, Bundle, Circle, Color, Commands, Component, DetectChanges, Entity,
    FixedUpdate, GizmoConfigGroup, GizmoConfigStore, Gizmos, Handle, IntoSystemConfigs, Mesh, Mut,
    Name, Query, Ref, Reflect, ReflectComponent, Res, ResMut, Startup, Transform, Update, Vec2,
    With,
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
    app.add_systems(FixedUpdate, (clear_force_lines, physics_update).chain());
    app.init_gizmo_group::<DebugCelestialGizmos>()
        .add_systems(Update, draw_lines);
}

#[derive(Default, Reflect, GizmoConfigGroup, AutoRegisterType)]
pub struct DebugCelestialGizmos {
    point_map: EntityHashMap<Entity, Vec<Vec2>>,
    force_lines: Vec<(Vec2, Vec2, Vec2)>,
}

#[derive(Component, Debug, Copy, Clone, Hash, Eq, PartialEq, Display, SmartDefault)]
pub enum CelestialMesh {
    #[display(fmt = "Standard({})", _0)]
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

#[derive(Debug)]
struct BodyPhysicsData {
    entity: Entity,
    mass: f32,
    pos: Vec2,
    velocity: Vec2,
    force_to_apply: Vec2,
    force_map: EntityHashMap<Entity, Vec2>,
}

impl From<PhysicsUpdateQueryDataReadOnlyItem<'_, '_>> for BodyPhysicsData {
    fn from(value: PhysicsUpdateQueryDataReadOnlyItem<'_, '_>) -> Self {
        Self {
            entity: value.entity,
            mass: value.mass.0,
            pos: value.transform.translation.truncate(),
            velocity: value.linear_velocity.0.into(),
            force_to_apply: default(),
            force_map: default(),
        }
    }
}

fn compute_forces(bodies: &mut Vec<BodyPhysicsData>) {
    const G: f32 = 6.67430e-11; // gravitational constant in m^3 kg^-1 s^-2

    let mut force_to_apply_map: EntityHashMap<Entity, Vec2> = default();
    let mut force_body_map: EntityHashMap<Entity, EntityHashMap<Entity, Vec2>> = default();

    for (i, body_i) in bodies.iter().enumerate() {
        for (j, body_j) in bodies.iter().enumerate() {
            if i != j {
                let r_ij = body_j.pos - body_i.pos;
                let distance = r_ij.length();
                if distance > 0.0 {
                    let force_magnitude = G * body_i.mass * body_j.mass / (distance * distance);
                    let force_direction = r_ij.normalize();
                    let force = force_direction * force_magnitude;
                    *force_to_apply_map.entry(body_i.entity).or_default() += force;
                    *force_body_map
                        .entry(body_i.entity)
                        .or_default()
                        .entry(body_j.entity)
                        .or_default() += force;
                }
            }
        }
    }

    for body in bodies.iter_mut() {
        body.force_to_apply = force_to_apply_map
            .remove(&body.entity)
            .unwrap_or_else(|| unreachable!());
        body.force_map = force_body_map
            .remove(&body.entity)
            .unwrap_or_else(|| unreachable!());
    }
}

const FORCE_SCALE: f32 = 10000.0;
fn physics_update(
    mut celestial_q: Query<PhysicsUpdateQueryData, With<CelestialBody>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (_config, debug_gizmos) = config_store.config_mut::<DebugCelestialGizmos>();
    let mut bodies = celestial_q
        .iter()
        .map(BodyPhysicsData::from)
        .collect::<Vec<_>>();
    compute_forces(&mut bodies);
    let body_map = bodies
        .into_iter()
        .map(|b| (b.entity, b))
        .collect::<HashMap<_, _>>();
    for mut item in celestial_q.iter_mut() {
        let Some(body) = body_map.get(&item.entity) else {
            unreachable!();
        };
        let force = body.force_to_apply * FORCE_SCALE;

        for (force_entity, force_body_force) in body.force_map.iter() {
            let Some(force_body) = body_map.get(force_entity) else {
                unreachable!();
            };
            debug_gizmos.force_lines.push((
                body.pos,
                force_body.pos,
                *force_body_force * FORCE_SCALE,
            ));
        }

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
    if let Some(((.., min_force), (.., max_force))) = debug_gizmos
        .config_ext
        .force_lines
        .iter()
        .map(|(a, b, force)| {
            (
                a,
                b,
                ordered_float::NotNan::new(force.length_squared())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
        })
        .minmax_by(|(.., a), (.., b)| a.cmp(b))
        .into_option()
    {
        const RATIO_FLOOR: f32 = 0.01;
        const RATIO_FLOOR_OFFSET: f32 = 1.0 - RATIO_FLOOR;
        for &(a, b, force) in debug_gizmos.config_ext.force_lines.iter() {
            let ratio = (force.length_squared() - *min_force) / (*max_force - *min_force);
            // TODO: only to test to make sure all lines are there
            // set ratio floor to 0.25 and scale everything to fit inside it
            let ratio = RATIO_FLOOR + ratio * RATIO_FLOOR_OFFSET;
            debug_gizmos.line_2d(a, b, Color::srgba(1.0, 1.0, 1.0, ratio));
        }
    }
}

fn clear_force_lines(mut config_store: ResMut<GizmoConfigStore>) {
    let (_config, debug_gizmos) = config_store.config_mut::<DebugCelestialGizmos>();
    debug_gizmos.force_lines.clear();
}
