// Derived from: https://github.com/jakobhellermann/bevy-inspector-egui/blob/f931976fcff47bdf4fb42e039ee5881c667a2e1f/crates/bevy-inspector-egui/examples/integrations/egui_dock.rs
// Original License:
//      MIT - https://github.com/jakobhellermann/bevy-inspector-egui/blob/f931976fcff47bdf4fb42e039ee5881c667a2e1f/LICENSE-MIT.md
//  or
//      Apache 2.0 - https://github.com/jakobhellermann/bevy-inspector-egui/blob/f931976fcff47bdf4fb42e039ee5881c667a2e1f/LICENSE-APACHE.md

// Derived from: https://github.com/urholaukkarinen/transform-gizmo/blob/00be178c38a09a6a8df2ae4f557b7a12fcdafe14/examples/bevy/src/gui.rs
// Original License:
//      MIT - https://github.com/urholaukkarinen/transform-gizmo/blob/00be178c38a09a6a8df2ae4f557b7a12fcdafe14/LICENSE-APACHE
//  or
//      Apache 2.0 - https://github.com/urholaukkarinen/transform-gizmo/blob/00be178c38a09a6a8df2ae4f557b7a12fcdafe14/LICENSE-MIT

use std::any::TypeId;

use avian2d::prelude::{Physics, PhysicsTime, TimestepMode};
use bevy::asset::{ReflectAsset, UntypedAssetId};
use bevy::log::tracing_subscriber::fmt::time;
use bevy::math::DQuat;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::render::camera::{CameraProjection, Viewport};
use bevy::utils::{HashMap, HashSet};
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::{bevy_egui, bevy_inspector, DefaultInspectorConfigPlugin};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiContexts, EguiSet};
use bevy_inspector_egui::bevy_inspector::{
    ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_inspector_egui::bevy_inspector::hierarchy::{hierarchy_ui, SelectedEntities};
use bevy_mod_picking::picking_core::PickingPluginsSettings;
use bevy_mod_picking::prelude::{HighlightPluginSettings, PickSelection};
use bevy_mod_picking::selection::SelectionPluginSettings;
use egui::Widget;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use smart_default::SmartDefault;
use transform_gizmo_bevy::{
    EnumSet, Gizmo, GizmoCamera, GizmoInteraction, GizmoMode, GizmoOptions, GizmoOrientation,
    GizmoResult, GizmoTarget, GizmoVisuals,
};
use transform_gizmo_bevy::mint::RowMatrix4;

use internal_proc_macros::{AutoRegisterType, RegisterTypeBinder};

use crate::game::camera::{MainCamera, MainCameraController};

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app
        //
        .configure_sets(Update, MainCameraController.run_if(is_game_view_focused))
        .insert_resource(UiState::new())
        .insert_resource(SelectionPluginSettings {
            click_nothing_deselect_all: false,
            ..default()
        })
        .insert_resource(HighlightPluginSettings { is_enabled: false })
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(
            PostUpdate,
            show_ui_system
                .before(EguiSet::ProcessOutput)
                .before(TransformSystem::TransformPropagate),
        )
        .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system))
        // TODO: not useful until 2d support is added
        // .add_systems(Update, set_gizmo_mode)
        .add_systems(PreUpdate, toggle_picking_enabled)
        .add_systems(Update, update_picking);
}

#[derive(Debug, Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Resource, Debug)]
struct UiState {
    state: DockState<EguiWindow>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
    gizmo_mode: GizmoMode,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![EguiWindow::Inspector, EguiWindow::Physics],
        );
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
        let [_game, _bottom] =
            tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            state,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
            gizmo_mode: GizmoMode::TranslateXY,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            gizmo_mode: self.gizmo_mode,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }

    fn state(&self) -> DockState<EguiWindow> {
        self.state.clone()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
    Physics,
}

#[derive(Debug)]
struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    gizmo_mode: GizmoMode,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        format!("{window:?}").into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();

                egui::Frame::none()
                    .outer_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        let label = match self.gizmo_mode {
                            GizmoMode::RotateX => "RotateX",
                            GizmoMode::RotateY => "RotateY",
                            GizmoMode::RotateZ => "RotateZ",
                            GizmoMode::RotateView => "RotateView",
                            GizmoMode::TranslateX => "TranslateX",
                            GizmoMode::TranslateY => "TranslateY",
                            GizmoMode::TranslateZ => "TranslateZ",
                            GizmoMode::TranslateXY => "TranslateXY",
                            GizmoMode::TranslateXZ => "TranslateXZ",
                            GizmoMode::TranslateYZ => "TranslateYZ",
                            GizmoMode::TranslateView => "TranslateView",
                            GizmoMode::ScaleX => "ScaleX",
                            GizmoMode::ScaleY => "ScaleY",
                            GizmoMode::ScaleZ => "ScaleZ",
                            GizmoMode::ScaleXY => "ScaleXY",
                            GizmoMode::ScaleXZ => "ScaleXZ",
                            GizmoMode::ScaleYZ => "ScaleYZ",
                            GizmoMode::ScaleUniform => "ScaleUniform",
                            GizmoMode::Arcball => "Arcball",
                        };
                        ui.label(format!("Mode: {label}"));
                    });

                let mut gizmo_options = self.world.resource_mut::<GizmoOptions>();
                // TODO: only shows gizmo in 3d so only allow translation for now
                gizmo_options.gizmo_modes = EnumSet::only(GizmoMode::TranslateView);

                let latest_gizmo_result = self
                    .world
                    .query::<&GizmoTarget>()
                    .iter(self.world)
                    .find_map(|target| target.latest_result());

                draw_gizmo_result(ui, latest_gizmo_result);
            }
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            },
            EguiWindow::Physics => {
                let mut time_physics = self.world.resource_mut::<Time<Physics>>();
                ui.heading("Physics");

                #[derive(Debug, PartialEq)]
                enum Speed {
                    Paused,
                    Single,
                    Double,
                    Triple,
                    Quad,
                }

                impl Speed {
                    fn get(time_physics: &Mut<Time<Physics>>) -> Self {
                        let is_paused = time_physics.is_paused();
                        let speed = time_physics.relative_speed();
                        match (is_paused, speed) {
                            (true, _) => Self::Paused,
                            (false, 1.0) => Self::Single,
                            (false, 2.0) => Self::Double,
                            (false, 3.0) => Self::Triple,
                            (false, 4.0) => Self::Quad,
                            _ => panic!("unknown speed {speed}"),
                        }
                    }
                    fn process(&self, time_physics: &mut Mut<Time<Physics>>) {
                        let is_paused = time_physics.is_paused();
                        let speed = time_physics.relative_speed();
                        match self {
                            Speed::Paused => {
                                if !is_paused {
                                    time_physics.pause();
                                }
                            }
                            Speed::Single => {
                                if is_paused || speed != 1.0 {
                                    time_physics.unpause();
                                    time_physics.set_relative_speed(1.0);
                                }
                            }
                            Speed::Double => {
                                if is_paused || speed != 2.0 {
                                    time_physics.unpause();
                                    time_physics.set_relative_speed(2.0);
                                    // TODO: should probably increase resolution
                                }
                            }
                            Speed::Triple => {
                                if is_paused || speed != 3.0 {
                                    time_physics.unpause();
                                    time_physics.set_relative_speed(3.0);
                                    // TODO: should probably increase resolution
                                }
                            }
                            Speed::Quad => {
                                if is_paused || speed != 4.0 {
                                    time_physics.unpause();
                                    time_physics.set_relative_speed(4.0);
                                    // TODO: should probably increase resolution
                                }
                            }
                        }
                    }
                }

                let mut speed = Speed::get(&time_physics);

                ui.radio_value(&mut speed, Speed::Paused, "|| Pause");
                ui.radio_value(&mut speed, Speed::Single, "> Play");
                ui.radio_value(&mut speed, Speed::Double, ">> Double");
                ui.radio_value(&mut speed, Speed::Triple, ">>> Triple");
                ui.radio_value(&mut speed, Speed::Quad, ">>>> Quad");

                speed.process(&mut time_physics);
            }
        }
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

#[derive(RegisterTypeBinder)]
pub struct Types;

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

// make camera only render to view not obstructed by UI
fn set_camera_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera, With<MainCamera>>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size = UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    // The desired viewport rectangle at its offset in "physical pixel space"
    let rect = physical_position + physical_size;

    let window_size = window.physical_size();
    // wgpu will panic if trying to set a viewport rect which has coordinates extending
    // past the size of the render target, i.e. the physical window in our case.
    // Typically this shouldn't happen- but during init and resizing etc. edge cases might occur.
    // Simply do nothing in those cases.
    if rect.x <= window_size.x && rect.y <= window_size.y {
        cam.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    }
}

fn toggle_picking_enabled(
    gizmo_targets: Query<&transform_gizmo_bevy::GizmoTarget>,
    mut picking_settings: ResMut<PickingPluginsSettings>,
) {
    // Picking is disabled when any of the gizmos is focused or active.

    picking_settings.is_enabled = gizmo_targets
        .iter()
        .all(|target| !target.is_focused() && !target.is_active());
}

fn update_picking(
    mut commands: Commands,
    targets: Query<(Entity, Ref<PickSelection>, Option<&GizmoTarget>), Changed<PickSelection>>,
) {
    // Continuously update entities based on their picking state

    for (entity, pick_selection, gizmo_target) in targets.iter() {
        if !pick_selection.is_changed() {
            continue;
        }
        let mut entity_cmd = commands.entity(entity);

        if pick_selection.is_selected {
            if gizmo_target.is_none() {
                entity_cmd.insert(GizmoTarget::default());
            }
            debug!("outline: {entity}");
            commands
                .entity(entity)
                .insert(crate::game::util::outline::Outline::default());
        } else {
            entity_cmd.remove::<GizmoTarget>();

            commands
                .entity(entity)
                .remove::<crate::game::util::outline::Outline>();
        }
    }
}

fn set_gizmo_mode(input: Res<ButtonInput<KeyCode>>, mut ui_state: ResMut<UiState>) {
    for (key, mode) in [
        (KeyCode::KeyR, GizmoMode::RotateZ),
        (KeyCode::KeyT, GizmoMode::TranslateXY),
        (KeyCode::KeyS, GizmoMode::ScaleXY),
    ] {
        if input.pressed(KeyCode::ControlLeft) && input.just_pressed(key) {
            ui_state.gizmo_mode = mode;
        }
    }
}

trait ToRowMatrix4F64 {
    fn convert_to_row_matrix4_f64(&self) -> RowMatrix4<f64>;
}

impl ToRowMatrix4F64 for Mat4 {
    fn convert_to_row_matrix4_f64(&self) -> RowMatrix4<f64> {
        RowMatrix4::<f64>::from(self.to_cols_array_2d().map(|c| c.map(|v| v as f64)))
    }
}

fn draw_gizmo_result(ui: &mut egui::Ui, gizmo_result: Option<GizmoResult>) {
    if let Some(result) = gizmo_result {
        let text = match result {
            GizmoResult::Rotation {
                axis,
                delta: _,
                total,
                is_view_axis: _,
            } => {
                format!(
                    "Rotation axis: ({:.2}, {:.2}, {:.2}), Angle: {:.2} deg",
                    axis.x,
                    axis.y,
                    axis.z,
                    total.to_degrees()
                )
            }
            GizmoResult::Translation { delta: _, total } => {
                format!(
                    "Translation: ({:.2}, {:.2}, {:.2})",
                    total.x, total.y, total.z,
                )
            }
            GizmoResult::Scale { total } => {
                format!("Scale: ({:.2}, {:.2}, {:.2})", total.x, total.y, total.z,)
            }
            GizmoResult::Arcball { delta: _, total } => {
                let (axis, angle) = DQuat::from(total).to_axis_angle();
                format!(
                    "Rotation axis: ({:.2}, {:.2}, {:.2}), Angle: {:.2} deg",
                    axis.x,
                    axis.y,
                    axis.z,
                    angle.to_degrees()
                )
            }
        };

        egui::Frame::none()
            .outer_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(text);
            });
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.to_string(), handle);
                }
            }
        });
    }
}

pub fn is_game_view_focused(ui_state: Res<UiState>) -> bool {
    let Some(_) = ui_state.state.focused_leaf() else {
        return false;
    };
    let mut state = ui_state.state();
    let Some((_, window)) = state.find_active_focused() else {
        return false;
    };
    match window {
        EguiWindow::GameView => true,
        _ => false,
    }
}
