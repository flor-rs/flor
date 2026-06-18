use crate::view::{View, ViewBox, ViewId};

/// This is a foundational trait used by view builders and converters.
pub trait ViewIdentity {
    fn identity(&self) -> ViewId;
}

impl<V: View> ViewIdentity for V {
    fn identity(&self) -> ViewId {
        View::view_id(self)
    }
}

impl ViewIdentity for ViewBox {
    fn identity(&self) -> ViewId {
        self.as_ref().view_id()
    }
}

/// Trait for converting a type into a ViewBox.
pub trait IntoView: ViewIdentity + Send + Sync + 'static {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static>;
}

impl<T: View + Send + Sync + 'static> IntoView for T {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(self) as Box<dyn View + Send + Sync + 'static>
    }
}

impl IntoView for ViewBox {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        self
    }
}

/// Trait for converting a type into an iterator of ViewBox.
pub trait IntoViewIter {
    type Iter: Iterator<Item = ViewBox>;
    fn into_view_iter(self) -> Self::Iter;
}

impl<T: IntoView> IntoViewIter for T {
    type Iter = std::iter::Once<ViewBox>;

    fn into_view_iter(self) -> Self::Iter {
        std::iter::once(self.into_view())
    }
}

impl IntoViewIter for Vec<ViewBox> {
    type Iter = std::vec::IntoIter<ViewBox>;

    fn into_view_iter(self) -> Self::Iter {
        self.into_iter()
    }
}
