use crate::signal::effect::updater_effect::create_updater;
use crate::view::View;

pub trait ZIndexBuilder {
    fn z_index(self, z_index: impl Fn() -> i32 + 'static) -> Self;
}

impl<V: View> ZIndexBuilder for V {
    fn z_index(self, z_index: impl Fn() -> i32 + 'static) -> Self {
        let view_id = self.view_id();
        let focus_index = create_updater(
            move || z_index(),
            move |v| view_id.update_z_index(v),
        );
        view_id.update_z_index(focus_index);
        self
    }
}
