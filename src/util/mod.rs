use bevy::prelude::App;

pub(crate) mod color_id;
pub(crate) mod color_material_manager;
pub(crate) mod prototype_mesh_manager;
pub(crate) mod ref_ext;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        color_id::plugin,
        color_material_manager::plugin,
        prototype_mesh_manager::plugin,
    ));
}
