use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;

pub trait ViewBuilder {
    fn views(self, views: impl IntoIterator<Item = Box<dyn View + Send + Sync + 'static>>) -> Self;
    fn push_view(self, view: impl View + Send + Sync + 'static) -> Self;
}

impl<V: View> ViewBuilder for V {
    fn views(self, views: impl IntoIterator<Item = Box<dyn View + Send + Sync + 'static>>) -> Self {
        let view_id = self.view_id();
        for view in views {
            VIEW_STORAGE.add_child(view_id, view);
        }
        self
    }

    fn push_view(self, view: impl View + Send + Sync + 'static) -> Self {
        VIEW_STORAGE.add_child(self.view_id(), Box::new(view));
        self
    }
}

#[macro_export]
macro_rules! view {
    ($x:expr) => {
        Box::new($x) as Box<dyn flor::view::View + Send + Sync + 'static>
    };
}

#[macro_export]
macro_rules! views {
    ( $( $x:expr ),* $(,)? ) => {
        vec![
            $( Box::new(($x)) as Box<dyn flor::view::View + Send + Sync + 'static> ),*
        ]
    };
}

pub trait IntoView {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static>;
    fn into_views(self) -> Vec<Box<dyn View + Send + Sync + 'static>>;
}

impl<T: View + Send + Sync + 'static> IntoView for T {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(self) as Box<dyn View + Send + Sync + 'static>
    }

    fn into_views(self) -> Vec<Box<dyn View + Send + Sync + 'static>> {
        vec![Box::new(self) as Box<dyn View + Send + Sync + 'static>]
    }
}
