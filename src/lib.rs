use avian2d::PhysicsPlugins;
use avian2d::prelude::{Gravity, PhysicsDebugPlugin};
use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    log::LogPlugin,
    prelude::*,
};
use bevy_frame_count_log_prefix::prelude::FrameCountLogPrefixPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use transform_gizmo_bevy::TransformGizmoPlugin;

use crate::game::camera::MainCameraBundle;

#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod screen;
mod ui;
mod util;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy Game Jam 5 Prototypes".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.3),
                    },
                    ..default()
                }),
        );

        // Third party plugins
        app.add_plugins((
            // multiline
            FrameCountLogPrefixPlugin,
            EguiPlugin,
            TransformGizmoPlugin,
            DefaultPickingPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ));

        // Disable standard gravity
        app.insert_resource(Gravity(Vec2::ZERO));

        // Add other plugins.
        app.add_plugins((
            // multiline
            util::plugin,
            game::plugin,
            screen::plugin,
            ui::plugin,
        ));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(MainCameraBundle::default());
}
