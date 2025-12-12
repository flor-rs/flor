use crate::signal::effect::updater_effect::create_updater;
use crate::view::View;

pub trait FocusIndexBuilder {
    fn focus_index(self, focus_index: impl Fn() -> u32 + 'static) -> Self;
}

impl<V: View> FocusIndexBuilder for V {
    fn focus_index(self, focus_index: impl Fn() -> u32 + 'static) -> Self {
        let view_id = self.view_id();
        let focus_index = create_updater(
            move || focus_index(),
            move |v| view_id.update_focus_index(v),
        );
        view_id.update_focus_index(focus_index);
        self
    }
}
