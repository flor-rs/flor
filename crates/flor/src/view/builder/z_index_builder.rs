use crate::signal::create_updater_with_id;
use crate::view::View;

pub trait ZIndexBuilder {
    fn z_index(self, z_index: impl Fn() -> i32 + 'static) -> Self;
}

impl<V: View> ZIndexBuilder for V {
    fn z_index(self, z_index: impl Fn() -> i32 + 'static) -> Self {
        let view_id = self.view_id();
        let (effect_id, focus_index) =
            create_updater_with_id(move || z_index(), move |v| view_id.set_z_index(v));
        view_id.pending_effect_id(effect_id);
        view_id.set_z_index(focus_index);
        self
    }
}
