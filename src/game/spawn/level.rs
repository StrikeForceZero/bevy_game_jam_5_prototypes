//! Spawn the main level by triggering other observers.

use avian2d::prelude::{AngularVelocity, CollisionMargin, Physics};
use bevy::color::palettes::css::RED;
use bevy::prelude::*;

use crate::game::platter::platter::{Platter, PlatterBundle};
use crate::game::platter::platter_object::{PlatterObject, PlatterObjectBundle};
use crate::game::util::debug_draw::DebugDrawGizmosSystemParam;
use crate::screen::Screen;
use crate::util::prototype_mesh_manager::{PrototypeMesh, PrototypeMeshId};
use crate::util::PrototypeManagerSystemParam;
use crate::util::ref_ext::RefExt;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.add_systems(Update, input);
    app.add_systems(Update, platter_object_added);
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

    commands
        .spawn((
            StateScoped(Screen::Playing),
            PlatterBundle::new(&mut prototype_manager_system_param, 100.0),
            CollisionMargin(0.5),
        ))
        .with_children(|parent| {
            parent.spawn((
                StateScoped(Screen::Playing),
                CollisionMargin(0.5),
                PlatterObjectBundle::new(&mut prototype_manager_system_param, RED, 10.0)
                    .with_transform(Transform::from_xyz(5.0, 5.0, 5.0)),
            ));
        });
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

fn platter_object_added(
    mut platter_objects_q: Query<
        (Entity, Ref<PlatterObject>, &Parent, &mut Transform),
        (Added<PlatterObject>, Without<Platter>),
    >,
    platter_q: Query<&Transform, With<Platter>>,
) {
    for (entity, platter_obj_ref, parent, mut transform) in platter_objects_q.iter_mut() {
        if !platter_obj_ref.is_added_or_changed() {
            continue;
        }
        let Some(platter) = platter_q.get(parent.get()).ok() else {
            continue;
        };
        transform.translation.z = platter.translation.z + 2.0;
        log::debug!("updating z for {entity} {}", transform.translation.z);
    }
}
