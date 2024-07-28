//! Spawn the main level by triggering other observers.

use avian2d::parry::utils::center;
use avian2d::prelude::{AngularVelocity, Collider, Physics};
use bevy::color::palettes::css::{BLUE, DARK_GRAY, RED};
use bevy::prelude::*;

use crate::game::platter::arm::PlatterArm;
use crate::game::platter::falling::{FallingSystemSet, SpawnFallingBlock};
use crate::game::platter::mesh::PlatterMeshOptionsObj;
use crate::game::platter::platter::{create_platter, CreatePlatterOptions, Platter};
use crate::game::platter::spawn::{SpawnArea, SpawnAreaBundle};
use crate::game::platter::value::InnerValue;
use crate::game::util::debug_draw::DebugDrawGizmosSystemParam;
use crate::game::util::mesh::{
    calculate_centroid, convex_hull, generate_subdivided_donut_split_vertices, rotate_point,
};
use crate::screen::Screen;
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId};
use crate::util::PrototypeManagerSystemParam;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.add_systems(Update, input);
    app.add_systems(Update, test_input.before(FallingSystemSet));
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    mut prototype_manager_system_param: PrototypeManagerSystemParam,
    mut debug_draw_gizmos: DebugDrawGizmosSystemParam,
    mut physics: ResMut<Time<Physics>>,
) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    // commands.trigger(SpawnPlayer);

    const PLATTER_RADIUS_OUTER: f32 = 250.0;
    const PLATTER_ARM_RADIUS: f32 = PLATTER_RADIUS_OUTER * 1.15;
    const PLATTER_RADIUS_INNER: f32 = 20. / 150.0 * PLATTER_RADIUS_OUTER;
    const PLATTER_ARM_RADIUS_CENTER: f32 = PLATTER_RADIUS_INNER * 0.9;

    let platter_mesh_options = PlatterMeshOptionsObj {
        inner_radius: PLATTER_RADIUS_INNER,
        outer_radius: PLATTER_RADIUS_OUTER,
        pie_cuts: 10,
        onion_layers: 20,
        ..default()
    };

    create_platter(
        commands.spawn(StateScoped(Screen::Playing)),
        &mut prototype_manager_system_param,
        CreatePlatterOptions {
            platter_mesh_options,
            ..default()
        },
    );

    commands.spawn((
        PlatterArm,
        Name::new("PlatterArm"),
        ColorMesh2dBundle {
            mesh: prototype_manager_system_param
                .meshes
                .add(Rectangle::new(5.0, PLATTER_ARM_RADIUS))
                .into(),
            material: prototype_manager_system_param.get_or_create_material(Color::from(DARK_GRAY)),
            transform: Transform::from_xyz(0.0, PLATTER_ARM_RADIUS / 2.0, 2.0),
            ..default()
        },
        Collider::rectangle(5.0, PLATTER_ARM_RADIUS),
    ));

    commands.spawn((
        Name::new("Center"),
        ColorMesh2dBundle {
            mesh: prototype_manager_system_param
                .meshes
                .add(Circle::new(PLATTER_ARM_RADIUS_CENTER))
                .into(),
            material: prototype_manager_system_param.get_or_create_material(Color::from(DARK_GRAY)),
            ..default()
        },
    ));

    commands.spawn(SpawnAreaBundle::new(platter_mesh_options));
}

fn input(
    physics_time: Res<Time<Physics>>,
    input: Res<ButtonInput<KeyCode>>,
    mut platter_q: Query<(Entity, &mut AngularVelocity), With<Platter>>,
    mut spawn: EventWriter<SpawnFallingBlock>,
) {
    for (_, mut angular_velocity) in platter_q.iter_mut() {
        let right = input.pressed(KeyCode::KeyE);
        let left = input.pressed(KeyCode::KeyQ);
        let velocity_delta = if left != right {
            10.0 * if right { 1.0 } else { -1.0 }
        } else if angular_velocity.0 >= 1.0 || angular_velocity.0 <= -1.0 {
            angular_velocity.0.signum() * -50.0
        } else {
            angular_velocity.0 = 0.0;
            continue;
        };
        angular_velocity.0 += velocity_delta * physics_time.delta_seconds();
        angular_velocity.0 = angular_velocity.0.clamp(-100.0, 100.0);
    }
}

fn test_input(
    input: Res<ButtonInput<KeyCode>>,
    platter_q: Query<Entity, With<Platter>>,
    mut spawn: EventWriter<SpawnFallingBlock>,
) {
    if input.just_pressed(KeyCode::Space) {
        for entity in platter_q.iter() {
            log::debug!("space: {entity}");
            spawn.send(SpawnFallingBlock {
                platter: entity,
                value: InnerValue::PurpleT,
            });
        }
    }
}
