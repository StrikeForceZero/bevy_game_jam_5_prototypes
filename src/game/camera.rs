use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::ui::inspector::Focus;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_systems(Update, pan_camera.in_set(MainCameraController));
    app.add_systems(Update, zoom_camera.in_set(MainCameraController));
}

#[derive(SystemSet, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MainCameraController;

#[derive(Component, Debug, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct MainCamera;

#[derive(RegisterTypeBinder)]
pub struct Types;

fn pan_camera(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<(Mut<Transform>, &OrthographicProjection), With<Camera>>,
    focused_entities: Query<(Entity, &Focus)>,
) {
    let speed = if input.pressed(KeyCode::ShiftLeft) {
        500.0
    } else {
        50.0
    };

    let mut unlock = false;

    if input.pressed(KeyCode::KeyW) {
        unlock = true;
        for (mut transform, projection) in camera_q.iter_mut() {
            transform.translation.y += speed * time.delta_seconds() * projection.scale;
        }
    }
    if input.pressed(KeyCode::KeyS) {
        unlock = true;
        for (mut transform, projection) in camera_q.iter_mut() {
            transform.translation.y -= speed * time.delta_seconds() * projection.scale;
        }
    }
    if input.pressed(KeyCode::KeyA) {
        unlock = true;
        for (mut transform, projection) in camera_q.iter_mut() {
            transform.translation.x -= speed * time.delta_seconds() * projection.scale;
        }
    }
    if input.pressed(KeyCode::KeyD) {
        unlock = true;
        for (mut transform, projection) in camera_q.iter_mut() {
            transform.translation.x += speed * time.delta_seconds() * projection.scale;
        }
    }

    if unlock {
        for (entity, focus) in focused_entities.iter() {
            if let Focus::Follow = focus {
                log::debug!("reverting focus {entity}");
                commands.entity(entity).insert(Focus::Normal);
            }
        }
    }
}

fn zoom_camera(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut event_scroll: EventReader<MouseWheel>,
    mut camera_q: Query<Mut<OrthographicProjection>, With<Camera>>,
) {
    for event in event_scroll.read() {
        let speed = if input.pressed(KeyCode::ShiftLeft) {
            500.0
        } else {
            100.0
        };

        let amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y,
        } * speed;
        for mut projection in camera_q.iter_mut() {
            let amount = if projection.scale <= 1.0 {
                amount * 0.25
            } else if projection.scale >= 3.0 {
                amount * 2.0
            } else {
                amount
            };
            projection.scale -= amount * time.delta_seconds();
            if projection.scale <= 0.0 {
                projection.scale = 0.01;
            }
        }
    }
}
