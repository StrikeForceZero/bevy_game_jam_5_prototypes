use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::query::QueryData;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::transform::systems::{propagate_transforms, sync_simple_transforms};
use smart_default::SmartDefault;
use transform_gizmo_bevy::GizmoCamera;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::util::ref_ext::RefExt;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app.add_event::<UnlockCamera>();
    app.configure_sets(Update, MainCameraControllerSet);
    app.add_systems(
        PostUpdate,
        camera_follow
            .after(sync_simple_transforms)
            .after(propagate_transforms),
    );
    app.add_systems(
        Update,
        (
            camera_zoom,
            on_unlock_camera,
            focus_changed,
            focus_removed,
            camera_pan,
        )
            .in_set(MainCameraControllerSet),
    );
}

#[derive(SystemSet, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MainCameraControllerSet;

#[derive(Event, Debug, Copy, Clone)]
pub enum UnlockCamera {
    MainCamera,
    Camera(Entity),
}

#[derive(Component, Debug, Clone, Default, PartialEq, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub enum Focus {
    #[default]
    Normal,
    Follow {
        target_camera: Entity,
    },
}

impl Focus {
    pub fn follow(target_camera: Entity) -> Self {
        Self::Follow { target_camera }
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq, Reflect, AutoRegisterType)]
#[reflect(Component)]
struct FollowingCache(
    /// Do not edit this outside of the focus_added, focus_removed systems
    EntityHashSet,
);

#[derive(Component, Debug, Default, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct PanningCamera;

#[derive(Component, Debug, Default, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct ZoomingCamera;

#[derive(Component, Debug, Default, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct FollowingCamera;

#[derive(Component, Debug, Default, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct MainCamera;

#[derive(Bundle, SmartDefault)]
pub struct CameraBundle {
    panning_camera: PanningCamera,
    zooming_camera: ZoomingCamera,
    following_camera: FollowingCamera,
    following_cache: FollowingCache,
    #[default(Name::new("Camera"))]
    name: Name,
    camera_2d_bundle: Camera2dBundle,
}

#[derive(Bundle, SmartDefault)]
pub struct MainCameraBundle {
    main_camera: MainCamera,
    panning_camera: PanningCamera,
    zooming_camera: ZoomingCamera,
    following_camera: FollowingCamera,
    following_cache: FollowingCache,
    #[default(GizmoCamera)]
    gizmo_camera: GizmoCamera,
    #[default(Name::new("MainCamera"))]
    name: Name,
    camera_2d_bundle: Camera2dBundle,
    #[default(IsDefaultUiCamera)]
    is_default_ui_camera: IsDefaultUiCamera,
}

#[derive(RegisterTypeBinder)]
pub struct Types;

#[derive(QueryData, Debug)]
#[query_data(mutable)]
pub struct CameraPanQuery<'w> {
    entity: Entity,
    transform: Mut<'w, Transform>,
    projection: &'w OrthographicProjection,
}

fn camera_pan(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<CameraPanQuery, With<PanningCamera>>,
    mut unlock_camera: EventWriter<UnlockCamera>,
) {
    let direction = {
        let mut direction = Vec2::ZERO;

        if input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        // normalize will return NaN if we call on a 0 length vector
        if direction == Vec2::ZERO {
            // return early
            return;
        }
        direction.normalize().extend(0.0)
    };

    let speed = if input.pressed(KeyCode::ShiftLeft) {
        500.0
    } else {
        50.0
    };
    let move_amount = direction * speed * time.delta_seconds();

    for mut item in camera_q.iter_mut() {
        item.transform.translation += move_amount * item.projection.scale;
        unlock_camera.send(UnlockCamera::Camera(item.entity));
    }
}

fn camera_zoom(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut event_scroll: EventReader<MouseWheel>,
    mut camera_q: Query<Mut<OrthographicProjection>, With<ZoomingCamera>>,
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
            let amount = if projection.scale <= 0.05 {
                amount * 0.001
            } else if projection.scale <= 0.25 {
                amount * 0.05
            } else if projection.scale <= 1.0 {
                amount * 0.25
            } else if projection.scale >= 3.0 {
                amount * 2.0
            } else {
                amount
            };
            projection.scale -= amount * time.delta_seconds();
            if projection.scale <= 0.0 {
                projection.scale = 0.001;
            }
        }
    }
}

#[derive(QueryData)]
pub struct FollowTargetCameraQuery<'w> {
    entity: Entity,
    focus: &'w Focus,
    global_transform: &'w GlobalTransform,
}

fn camera_follow(
    follow_targets: Query<FollowTargetCameraQuery, Without<FollowingCamera>>,
    mut camera_q: Query<(Entity, Mut<Transform>, &FollowingCache), With<FollowingCamera>>,
) {
    for (camera_entity, mut transform, following_cache) in camera_q.iter_mut() {
        let follow_target_positions: Vec<Vec3> = following_cache
            .0
            .iter()
            .filter_map(|&follow_target| {
                follow_targets
                    .get(follow_target)
                    .ok()
                    .and_then(|item| match *item.focus {
                        Focus::Normal => None,
                        Focus::Follow { target_camera } => {
                            if target_camera == camera_entity {
                                Some(item.global_transform.translation())
                            } else {
                                None
                            }
                        }
                    })
            })
            .collect::<Vec<_>>();

        let transform_count = follow_target_positions.len();

        if transform_count > 0 {
            let average_translation = follow_target_positions
                .into_iter()
                .fold(Vec3::ZERO, |sum, next| sum + next)
                / transform_count as f32;
            transform.translation = average_translation;
        }
    }
}

fn on_unlock_camera(
    mut commands: Commands,
    mut events: EventReader<UnlockCamera>,
    mut main_camera_q: Query<(Entity, Mut<FollowingCache>), With<MainCamera>>,
    mut other_camera_q: Query<(Entity, Mut<FollowingCache>), (With<Camera>, Without<MainCamera>)>,
) {
    for &event in events.read() {
        // resolve the target camera
        let mut following_cache = match event {
            UnlockCamera::MainCamera => {
                let Some((_, following_cache)) = main_camera_q.get_single_mut().ok() else {
                    panic!("failed to find MainCamera camera");
                };
                following_cache
            }
            UnlockCamera::Camera(target) => {
                let Some((_, following_cache)) = other_camera_q
                    .get_mut(target)
                    .or_else(|_| main_camera_q.get_mut(target))
                    .ok()
                else {
                    panic!("failed to find target camera {target}");
                };
                following_cache
            }
        };
        if !following_cache.0.is_empty() {
            log::debug!("unlocking camera {event:?}");
            // clear its following
            for &focus_entity in following_cache.0.iter() {
                commands.entity(focus_entity).remove::<Focus>();
            }
            following_cache.0.clear();
        }
    }
}

fn focus_changed(
    changed_focus: Query<(Entity, Ref<Focus>), Changed<Focus>>,
    mut camera_q: Query<(Entity, Mut<FollowingCache>), With<FollowingCamera>>,
) {
    for (entity, focus) in changed_focus.iter() {
        if !focus.is_added_or_changed() {
            continue;
        }
        match *focus {
            Focus::Normal => {
                for (_, mut following_cache) in camera_q.iter_mut() {
                    following_cache.0.remove(&entity);
                }
            }
            Focus::Follow { target_camera } => {
                for (camera, mut following_cache) in camera_q.iter_mut() {
                    if camera == target_camera {
                        // don't remove if same
                        continue;
                    }
                    following_cache.0.remove(&entity);
                }
                let Some((_, mut following_cache)) = camera_q.get_mut(target_camera).ok() else {
                    panic!("failed to find target camera {target_camera}");
                };
                following_cache.0.insert(entity);
            }
        }
    }
}

fn focus_removed(
    mut removed_focus: RemovedComponents<Focus>,
    mut camera_q: Query<Mut<FollowingCache>, With<FollowingCamera>>,
) {
    let removed = removed_focus.read().collect::<EntityHashSet>();
    if removed.is_empty() {
        return;
    }
    for mut following_cache in camera_q.iter_mut() {
        following_cache.0.retain(|entity| !removed.contains(entity));
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::InputPlugin;

    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct TestScene {
        main_camera: Entity,
        camera: Entity,
        item1: Entity,
        item2: Entity,
    }

    #[derive(Debug, Clone, Default, PartialEq)]
    struct CurrentState {
        main_camera: EntityHashSet,
        main_camera_transform: Transform,
        camera: EntityHashSet,
        camera_transform: Transform,
        item1: Option<Focus>,
        item1_transform: Transform,
        item2: Option<Focus>,
        item2_transform: Transform,
    }

    impl CurrentState {
        fn get(app: &mut App, test_scene: TestScene) -> CurrentState {
            app.world_mut().run_system_once_with(
                test_scene,
                |test_scene: In<TestScene>,
                 main_camera_q: Query<(&FollowingCache, &Transform), With<MainCamera>>,
                 camera_q: Query<(&FollowingCache, &Transform), Without<MainCamera>>,
                 focus_q: Query<(Option<&Focus>, &Transform)>| {
                    let (main_camera, &main_camera_transform) = main_camera_q
                        .get(test_scene.main_camera)
                        .expect("expected MainCamera camera");
                    let (camera, &camera_transform) = camera_q
                        .get(test_scene.camera)
                        .expect("expected other camera");
                    let (item1, &item1_transform) =
                        focus_q.get(test_scene.item1).expect("expected item1");
                    let (item2, &item2_transform) =
                        focus_q.get(test_scene.item2).expect("expected item2");
                    Self {
                        main_camera: main_camera.0.clone(),
                        main_camera_transform,
                        camera: camera.0.clone(),
                        camera_transform,
                        item1: item1.cloned(),
                        item1_transform,
                        item2: item2.cloned(),
                        item2_transform,
                    }
                },
            )
        }
    }

    fn setup() -> (App, TestScene) {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            InputPlugin,
            TransformPlugin,
            HierarchyPlugin,
            plugin,
        ));
        let test_scene = app.world_mut().run_system_once(|mut commands: Commands| {
            let main_camera = commands.spawn(MainCameraBundle::default()).id();
            let camera = commands.spawn(CameraBundle::default()).id();
            let item1 = commands.spawn(SpatialBundle::default()).id();
            let item2 = commands.spawn(SpatialBundle::default()).id();
            TestScene {
                main_camera,
                camera,
                item1,
                item2,
            }
        });
        app.update();
        (app, test_scene)
    }

    #[test]
    fn test_focus_add() {
        let (mut app, test_scene) = setup();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState::default(),
            "expected initial state"
        );
        app.world_mut()
            .commands()
            .entity(test_scene.item1)
            .insert(Focus::Normal);
        app.update();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState {
                item1: Some(Focus::Normal),
                ..default()
            },
            "expected only changed to item1"
        );
        app.world_mut()
            .commands()
            .entity(test_scene.item2)
            .insert(Focus::follow(test_scene.main_camera));
        app.update();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState {
                main_camera: EntityHashSet::from_iter([test_scene.item2]),
                item1: Some(Focus::Normal),
                item2: Some(Focus::follow(test_scene.main_camera)),
                ..default()
            },
            "expected changes to only main_camera and item2"
        );
        app.world_mut()
            .commands()
            .entity(test_scene.item1)
            .insert(Focus::follow(test_scene.camera));
        app.world_mut()
            .commands()
            .entity(test_scene.item2)
            .insert(Focus::Normal);
        app.update();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState {
                camera: EntityHashSet::from_iter([test_scene.item1]),
                item1: Some(Focus::follow(test_scene.camera)),
                item2: Some(Focus::Normal),
                ..default()
            },
            "expected cameras to swap and items to swap states"
        );
        app.world_mut()
            .commands()
            .entity(test_scene.item2)
            .insert(Focus::follow(test_scene.camera));
        app.update();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState {
                camera: EntityHashSet::from_iter([test_scene.item2, test_scene.item1]),
                item1: Some(Focus::follow(test_scene.camera)),
                item2: Some(Focus::follow(test_scene.camera)),
                ..default()
            },
            "expected camera to follow both items"
        );
    }
    #[test]
    fn test_follow() {
        let (mut app, test_scene) = setup();
        let item1_transform = Transform::from_translation(Vec3::Y);
        app.world_mut()
            .commands()
            .entity(test_scene.item1)
            .insert(Focus::Normal)
            .insert(item1_transform);
        let item2_transform = Transform::from_translation(Vec3::X);
        app.world_mut()
            .commands()
            .entity(test_scene.item2)
            .insert(Focus::follow(test_scene.main_camera))
            .insert(item2_transform);
        app.world_mut()
            .commands()
            .entity(test_scene.main_camera)
            // set arbitrary transform that should be ignored because following
            .insert(Transform::from_xyz(999.0, 0.0, 0.0));
        app.update();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState {
                main_camera: EntityHashSet::from_iter([test_scene.item2]),
                main_camera_transform: item2_transform,
                item1: Some(Focus::Normal),
                item2: Some(Focus::follow(test_scene.main_camera)),
                item1_transform,
                item2_transform,
                ..default()
            },
            "expected main_camera transform to match item2"
        );
        // swap cameras
        app.world_mut()
            .commands()
            .entity(test_scene.item2)
            .insert(Focus::follow(test_scene.camera));
        app.update();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState {
                camera: EntityHashSet::from_iter([test_scene.item2]),
                main_camera_transform: item2_transform,
                camera_transform: item2_transform,
                item1: Some(Focus::Normal),
                item2: Some(Focus::follow(test_scene.camera)),
                item1_transform,
                item2_transform,
                ..default()
            },
            "expected other camera to snap to item2",
        );
        app.world_mut().send_event(UnlockCamera::MainCamera);
        app.update();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState {
                camera: EntityHashSet::from_iter([test_scene.item2]),
                main_camera_transform: item2_transform,
                camera_transform: item2_transform,
                item1: Some(Focus::Normal),
                item2: Some(Focus::follow(test_scene.camera)),
                item1_transform,
                item2_transform,
                ..default()
            },
            "expected no changes after unlock main camera",
        );
        let camera_transform = Transform::from_xyz(100.0, 0.0, 0.0);
        app.world_mut()
            .send_event(UnlockCamera::Camera(test_scene.camera));
        app.world_mut()
            .commands()
            .entity(test_scene.camera)
            .insert(camera_transform);
        app.update();
        assert_eq!(
            CurrentState::get(&mut app, test_scene),
            CurrentState {
                main_camera_transform: item2_transform,
                camera_transform,
                item1: Some(Focus::Normal),
                item1_transform,
                item2_transform,
                ..default()
            },
            "expected other camera to unlock and move to new position",
        );
    }
}
