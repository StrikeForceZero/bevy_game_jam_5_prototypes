//! Spawn the main level by triggering other observers.

use avian2d::prelude::{AngularVelocity, Physics};
use bevy::color::palettes::css::{BLUE, RED};
use bevy::prelude::*;

use crate::game::platter::mesh::PlatterMeshOptionsObj;
use crate::game::platter::platter::{create_platter, CreatePlatterOptions, Platter};
use crate::game::util::debug_draw::DebugDrawGizmosSystemParam;
use crate::game::util::mesh::{calculate_centroid, generate_subdivided_donut_split_vertices};
use crate::screen::Screen;
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId};
use crate::util::PrototypeManagerSystemParam;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.add_systems(Update, input);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Debug)]
struct Foo(f32);

impl PrototypeMesh for Foo {
    fn get_id(&self) -> PrototypeMeshId {
        format!("{self:?}").into()
    }

    fn get_mesh(&self) -> Mesh {
        Circle::new(self.0).into()
    }
}

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

    create_platter(
        commands.spawn(StateScoped(Screen::Playing)),
        &mut prototype_manager_system_param,
        CreatePlatterOptions {
            platter_mesh_options: PlatterMeshOptionsObj {
                inner_radius: 20.0,
                outer_radius: 125.0,
                pie_cuts: 10,
                onion_layers: 20,
                ..default()
            },
            ..default()
        },
    );

    // debug_draw_segments(&mut debug_draw_gizmos);
}

// visually generate_subdivided_donut_split_vertices
fn debug_draw_segments(debug_draw_gizmos: &mut DebugDrawGizmosSystemParam) {
    for (ax, slices) in generate_subdivided_donut_split_vertices(20.0, 125.0, 32, 64, 10, 20, true)
        .into_iter()
        .enumerate()
    {
        for (bx, segment) in slices.into_iter().enumerate() {
            let color = if (ax % 2) ^ (bx % 2) != 0 { RED } else { BLUE };
            let center = calculate_centroid(&segment);
            for point in segment {
                let direction_to_origin = (point - center) * 0.05;
                // shrink
                let point = point - direction_to_origin;
                debug_draw_gizmos
                    .get()
                    .scope(format!(
                        "debug generate_subdivided_donut_split_vertices {ax} {bx}"
                    ))
                    .add_color_point(point, color);
            }
        }
    }
}

fn input(
    physics_time: Res<Time<Physics>>,
    input: Res<ButtonInput<KeyCode>>,
    mut platter_q: Query<&mut AngularVelocity, With<Platter>>,
) {
    for mut angular_velocity in platter_q.iter_mut() {
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
