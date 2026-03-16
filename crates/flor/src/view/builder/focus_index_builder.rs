use crate::view::View;

pub trait FocusIndexBuilder {
    fn focus_scope(self, focus_scope: u32) -> Self;
    fn focus_index(self, focus_index: u32) -> Self;
}

impl<V: View> FocusIndexBuilder for V {
    fn focus_scope(self, focus_scope: u32) -> Self {
        let view_id = self.view_id();
        view_id.init_focus_scope(focus_scope);
        self
    }
    fn focus_index(self, focus_index: u32) -> Self {
        let view_id = self.view_id();
        view_id.init_focus_index(focus_index);
        self
    }
}
