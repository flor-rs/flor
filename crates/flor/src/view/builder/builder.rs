use crate::view::{IntoView, IntoViewIter, View, VIEW_STORAGE};

pub trait ViewBuilder {
    //: HasViewId + Sized {
    fn views(self, views: impl IntoViewIter) -> Self;

    fn push_view(self, view: impl IntoView) -> Self;
}

impl<T: View> ViewBuilder for T {
    fn views(self, views: impl IntoViewIter) -> Self {
        let view_id = self.view_id();
        VIEW_STORAGE.add_childs(view_id, views);
        self
    }

    fn push_view(self, view: impl IntoView) -> Self {
        VIEW_STORAGE.add_child(self.view_id(), view.into_view());
        self
    }
}

// impl<V: HasViewId> ViewBuilder for V {}

#[macro_export]
macro_rules! view {
    ($x:expr) => {
        $crate::view::IntoView::into_view($x)
    };
}

#[macro_export]
macro_rules! views {
    ( $( $x:expr ),* $(,)? ) => {
        vec![
            $(
                $crate::view::IntoView::into_view($x)
            ),*
        ]
    };
}
