use crate::view::handler::OnClickHandler;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;

pub trait EventBuilder {
    fn on_click(self, on_click: impl Into<OnClickHandler>) -> Self;
}

impl<V: View> EventBuilder for V {
    fn on_click(self, on_click: impl Into<OnClickHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_click_handler = Some(on_click.into());
        }
        self
    }
}
