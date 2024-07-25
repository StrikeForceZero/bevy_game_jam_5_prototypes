use avian2d::collision::Collider;
use avian2d::dynamics::integrator::IntegrationSet;
use avian2d::math::Vector;
use avian2d::prelude::{
    AngularVelocity, ExternalForce, LinearVelocity, Mass, MassPropertiesBundle, Physics,
    PhysicsSchedule, PhysicsStepSet, RigidBody, SpeculativeMargin, SweptCcd,
};
use bevy::color::palettes::basic::RED;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use smart_default::SmartDefault;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::platter::platter::{Platter, PlatterRadius};
use crate::game::util::force::calculate_centrifugal_force;
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId};
use crate::util::PrototypeManagerSystemParam;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_systems(
        PhysicsSchedule,
        update_forces
            .before(IntegrationSet::Velocity)
            .before(PhysicsStepSet::Solver)
            .after(PhysicsStepSet::NarrowPhase),
    );
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterObject;

#[derive(Component, Debug, Default, Copy, Clone, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PlatterObjectRadius(f32);

impl PrototypeMesh for PlatterObjectRadius {
    fn get_id(&self) -> PrototypeMeshId {
        format!("{self:?}").into()
    }

    fn get_mesh(&self) -> Mesh {
        Circle::new(self.0).into()
    }
}

#[derive(Bundle, SmartDefault, Clone)]
pub struct PlatterObjectBundle {
    #[default(Name::new("PlatterObject"))]
    name: Name,
    platter_object: PlatterObject,
    platter_radius: PlatterRadius,
    color_mesh_2d_bundle: ColorMesh2dBundle,
    collider: Collider,
    mass_properties_bundle: MassPropertiesBundle,
    #[default(RigidBody::Dynamic)]
    rigid_body: RigidBody,
    linear_velocity: LinearVelocity,
    #[default(SpeculativeMargin(0.0001))]
    speculative_margin: SpeculativeMargin,
    swept_ccd: SweptCcd,
}

impl PlatterObjectBundle {
    pub fn new(
        prototype_manager_system_param: &mut PrototypeManagerSystemParam,
        color: impl Into<Color> + Copy,
        radius: f32,
    ) -> Self {
        let platter_object_radius = PlatterObjectRadius(radius);
        let collider = Collider::circle(radius);
        let color_mesh_2d_bundle = prototype_manager_system_param
            .get_or_create_color_mesh_2d(&platter_object_radius, color);
        Self {
            mass_properties_bundle: MassPropertiesBundle::new_computed(&collider, 1.0),
            collider,
            color_mesh_2d_bundle,
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
    platter_radius: &'w PlatterRadius,
    angular_velocity: &'w AngularVelocity,
    global_transform: &'w GlobalTransform,
}

#[derive(QueryData)]
struct PlatterObjectQuery<'w> {
    entity: Entity,
    parent: &'w Parent,
    mass: &'w Mass,
    external_force: &'w ExternalForce,
    global_transform: &'w GlobalTransform,
    transform: &'w Transform,
}

#[derive(SystemParam)]
struct UpdateForceSystemParam<'w, 's> {
    platter_q: Query<'w, 's, PlatterQuery<'static>, With<Platter>>,
    platter_obj_q: Query<'w, 's, PlatterObjectQuery<'static>, With<PlatterObject>>,
}

fn update_forces(
    physics_time: Res<Time<Physics>>,
    mut commands: Commands,
    params: UpdateForceSystemParam,
) {
    for platter_object in params.platter_obj_q.iter() {
        let Some(platter) = params.platter_q.get(platter_object.parent.get()).ok() else {
            continue;
        };
        let pos = platter_object.transform.translation.truncate();
        let distance_from_center = pos.length();
        let force_magnitude = calculate_centrifugal_force(
            platter_object.mass.0,
            platter.angular_velocity.0,
            distance_from_center.abs(),
        );
        let direction = pos.normalize();
        let force = direction * force_magnitude * physics_time.delta_seconds();
        if force == Vec2::ZERO {
            commands
                .entity(platter_object.entity)
                .insert(ExternalForce::new(Vec2::ZERO));
        } else {
            commands
                .entity(platter_object.entity)
                .insert(ExternalForce::new(Vec2::new(
                    force.x + platter_object.external_force.x,
                    force.y + platter_object.external_force.y,
                )));
        }
    }
}
