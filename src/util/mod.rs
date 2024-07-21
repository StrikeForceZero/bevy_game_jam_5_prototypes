use bevy::prelude::App;

mod color_id;
mod color_material_manager;
mod prototype_mesh_manager;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        color_id::plugin,
        color_material_manager::plugin,
        prototype_mesh_manager::plugin,
    ));
}
