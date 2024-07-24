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

use avian2d::math::Vector;
use avian2d::prelude::{LinearVelocity, Physics, PhysicsTime, TimestepMode};
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
use bevy_inspector_egui::bevy_inspector::hierarchy::{
    hierarchy_ui, SelectedEntities, SelectionMode,
};
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
use crate::game::orbital::celestial::CelestialBody;
use crate::game::spawn::level::SpawnLevel;
use crate::screen::Screen;

pub(crate) fn plugin(app: &mut App) {
    Types.register_types(app);
    app
        //
        .add_event::<Select>()
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
        .add_systems(
            Update,
            (selected_removed, update_selected, update_picking, on_select).chain(),
        );
}

#[derive(Event, Debug, Clone, Reflect, AutoRegisterType)]
pub struct Select(Entity);

#[derive(Component, Debug, Clone, Default, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub struct Selected;

#[derive(Component, Debug, Clone, Default, Reflect, AutoRegisterType)]
#[reflect(Component)]
pub enum Focus {
    #[default]
    Normal,
    Follow,
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
    last_selection_action: Option<(SelectionMode, Entity)>,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![
                EguiWindow::Inspector,
                EguiWindow::Physics,
                EguiWindow::GameState,
            ],
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
            last_selection_action: None,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            gizmo_mode: self.gizmo_mode,
            last_selection_action: &mut self.last_selection_action,
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
    GameState,
}

#[derive(Debug)]
struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    gizmo_mode: GizmoMode,
    last_selection_action: &'a mut Option<(SelectionMode, Entity)>,
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

                let Some((entity, has_velocity, latest_gizmo_result)) = self
                    .world
                    .query::<(Entity, Has<LinearVelocity>, &GizmoTarget)>()
                    .iter(self.world)
                    .find_map(|(entity, has_velocity, target)| {
                        target
                            .latest_result()
                            .map(|result| (entity, has_velocity, result))
                    })
                else {
                    return;
                };

                if has_velocity {
                    if let GizmoResult::Translation { delta, .. } = &latest_gizmo_result {
                        self.world
                            .commands()
                            .entity(entity)
                            .insert(LinearVelocity(Vector::new(delta.x as f32, delta.y as f32)));
                    }
                }

                draw_gizmo_result(ui, Some(latest_gizmo_result));
            }
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => {
                match *self.selection {
                    InspectorSelection::Entities => {
                        {
                            let last_action = self.selected_entities.last_action();
                            let last_action_changed = {
                                type SM = SelectionMode;
                                // SelectionMode doesn't impl PartialEq
                                match (*self.last_selection_action, last_action) {
                                    (None, None) => false,
                                    (Some((SM::Replace, a)), Some((SM::Replace, b))) => a != b,
                                    (Some((SM::Extend, a)), Some((SM::Extend, b))) => a != b,
                                    (Some((SM::Add, a)), Some((SM::Add, b))) => a != b,
                                    _ => true,
                                }
                            };
                            if last_action_changed {
                                *self.last_selection_action = last_action;
                                log::debug!("inspector entity selection changed {last_action:?}");
                                if let Some((selection_mode, selection_action_entity)) = last_action
                                {
                                    let mut selected_entities_to_remove = self
                                        .world
                                        .query_filtered::<Entity, With<Selected>>()
                                        .iter(self.world)
                                        .collect::<HashSet<_>>();
                                    for current_entity in self.selected_entities.iter() {
                                        let Some((has_focus, has_selected)) = self
                                            .world
                                            .query::<(Has<Focus>, Has<Selected>)>()
                                            .get(self.world, current_entity)
                                            .ok()
                                        else {
                                            continue;
                                        };
                                        let mut commands = self.world.commands();
                                        let mut entity_commands = commands.entity(current_entity);
                                        if let SelectionMode::Replace = selection_mode {
                                            // clear any not in current set
                                            if current_entity != selection_action_entity {
                                                entity_commands.remove::<Selected>();
                                                continue;
                                            }
                                        };
                                        selected_entities_to_remove.remove(&current_entity);
                                        // add components if needed
                                        if !has_selected {
                                            entity_commands.insert(Selected);
                                        }
                                        if !has_focus {
                                            log::debug!("adding follow {current_entity}");
                                            entity_commands.insert(Focus::Follow);
                                        }
                                    }
                                    for entity in selected_entities_to_remove {
                                        self.world.commands().entity(entity).remove::<Selected>();
                                    }
                                } else {
                                    // clear all
                                    for entity in self
                                        .world
                                        .query_filtered::<Entity, Or<(With<Selected>, With<Focus>)>>()
                                        .iter(self.world).collect::<Vec<_>>()
                                    {
                                        self.world.commands().entity(entity).remove::<Selected>().remove::<Focus>();
                                    }
                                }
                            }
                        }
                        match self.selected_entities.as_slice() {
                            &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                            entities => ui_for_entities_shared_components(self.world, entities, ui),
                        }
                    }
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
                }
            }
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
            EguiWindow::GameState => {
                if ui.button("Reload World").clicked() {
                    self.world.resource_mut::<Time<Physics>>().pause();
                    let bodies = self
                        .world
                        .query_filtered::<Entity, With<CelestialBody>>()
                        .iter(self.world)
                        .collect::<Vec<_>>();
                    for entity in bodies {
                        self.world.commands().entity(entity).despawn_recursive();
                    }
                    self.world.commands().trigger(SpawnLevel);
                }
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

fn on_select(
    mut ui_state: ResMut<UiState>,
    mut select_evr: EventReader<Select>,
    selected: Query<Entity, With<Selected>>,
) {
    let selected_events = select_evr
        .read()
        .map(|&Select(entity)| entity)
        .collect::<Vec<_>>();
    if !selected_events.is_empty() {
        debug!("Select event received for {selected_events:?}",);
        debug!(
            "setting select for {:?}",
            selected.iter().collect::<Vec<_>>()
        );
        if let Some((SelectionMode::Replace, selected_entity)) = ui_state.last_selection_action {
            if selected.contains(selected_entity) {
                debug!("selection action already registered by inspector: {selected_entity}");
                return;
            }
        }
        ui_state.selected_entities.clear();
        for entity in selected.iter() {
            ui_state.selected_entities.select_maybe_add(entity, true);
        }
    }
}

fn update_selected(
    mut commands: Commands,
    mut main_camera_transform: Query<Mut<Transform>, With<MainCamera>>,
    newly_selected: Query<(Entity, Ref<Selected>, Has<Focus>), Added<Selected>>,
    all_selected: Query<(Entity, &GlobalTransform, &Focus), (With<Selected>, With<Focus>)>,
    mut select_evw: EventWriter<Select>,
) {
    for (selected_entity, selected_ref, has_focus) in newly_selected.iter() {
        if !selected_ref.is_added() {
            continue;
        }
        debug!("update selected: {selected_entity}");
        let mut entity_commands = commands.entity(selected_entity);
        if !has_focus {
            entity_commands.insert(Focus::Normal);
        }
        entity_commands.insert(PickSelection { is_selected: true });
        // delay the inspector selection by a frame
        select_evw.send(Select(selected_entity));
    }

    {
        // make camera follow selection
        let selected_transforms = all_selected
            .iter()
            .filter_map(|(_, t, focus)| match focus {
                Focus::Normal => None,
                Focus::Follow => Some(t),
            })
            .collect::<Vec<_>>();
        let transform_count = selected_transforms.len();
        if transform_count > 0 {
            let average_translation = selected_transforms
                .into_iter()
                .fold(Vec3::ZERO, |sum, next| sum + next.translation())
                / transform_count as f32;
            for mut camera_transform in main_camera_transform.iter_mut() {
                camera_transform.translation = average_translation;
            }
        }
    }
}

fn selected_removed(
    mut commands: Commands,
    mut removed_selected: RemovedComponents<Selected>,
    mut ui_state: ResMut<UiState>,
) {
    for removed in removed_selected.read() {
        debug!("selected_removed: {removed}");
        commands
            .entity(removed)
            .remove::<Focus>()
            .try_insert(PickSelection { is_selected: false });
        ui_state.selected_entities.remove(removed);
    }
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
        debug!(
            "Changed<PickSelection>: {entity} is_selected: {}",
            pick_selection.is_selected
        );
        let mut entity_cmd = commands.entity(entity);

        if pick_selection.is_selected {
            if gizmo_target.is_none() {
                entity_cmd.insert(GizmoTarget::default());
            }

            entity_cmd
                .insert(crate::game::util::outline::Outline::default())
                .insert(Selected);
        } else {
            entity_cmd
                .remove::<GizmoTarget>()
                .remove::<crate::game::util::outline::Outline>()
                .remove::<Selected>();
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
