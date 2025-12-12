use crate::view::style::layout::LayoutStateSelector;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;

pub trait LayoutBuilder {
    fn layout(self, style: impl Fn(LayoutStateSelector) -> LayoutStateSelector) -> Self;
}

impl<T: View> LayoutBuilder for T {
    fn layout(self, style: impl Fn(LayoutStateSelector) -> LayoutStateSelector) -> Self {
        let states = VIEW_STORAGE
            .states
            .read();

        let style = {
            let mut view_state = states
                .get(self.view_id())
                .expect(&format!("view[{}] not found ViewState",self.view_id()))
                .read();
            style(view_state.layout_style.clone())
        };

        let mut view_state = states
            .get(self.view_id())
            .expect(&format!("view[{}] not found ViewState",self.view_id()))
            .write();
        view_state.layout_style = style;
        self
    }
}
