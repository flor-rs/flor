use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use std::sync::Arc;

pub trait EventBuilder {
    fn on_click(self, on_click: impl Fn() + Send + Sync + 'static) -> Self;
}

impl<V: View> EventBuilder for V {
    fn on_click(self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.states.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.click_handler = Some(Arc::new(on_click));
        }
        self
    }
}
