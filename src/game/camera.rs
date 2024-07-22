use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, pan_camera);
    app.add_systems(Update, zoom_camera);
}

fn pan_camera(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<Mut<Transform>, With<Camera>>,
) {
    let speed = if input.pressed(KeyCode::ShiftLeft) {
        500.0
    } else {
        50.0
    };

    if input.pressed(KeyCode::KeyW) {
        for mut transform in camera_q.iter_mut() {
            transform.translation.y += speed * time.delta_seconds();
        }
    }
    if input.pressed(KeyCode::KeyS) {
        for mut transform in camera_q.iter_mut() {
            transform.translation.y -= speed * time.delta_seconds();
        }
    }
    if input.pressed(KeyCode::KeyA) {
        for mut transform in camera_q.iter_mut() {
            transform.translation.x -= speed * time.delta_seconds();
        }
    }
    if input.pressed(KeyCode::KeyD) {
        for mut transform in camera_q.iter_mut() {
            transform.translation.x += speed * time.delta_seconds();
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
        }
    }
}
