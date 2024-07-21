use bevy_app::App;

pub trait RegisterTypeBinder {
    fn register_types(self, app: &mut App);
}
