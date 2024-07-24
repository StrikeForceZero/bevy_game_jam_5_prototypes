use bevy::prelude::{DetectChanges, Ref};

pub trait RefExt {
    fn is_added_or_changed(&self) -> bool;
}

impl<T> RefExt for Ref<'_, T> {
    fn is_added_or_changed(&self) -> bool {
        self.is_added() || self.is_changed()
    }
}
