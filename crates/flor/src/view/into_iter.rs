use crate::view::{View, ViewBox};

pub trait IntoViewIter {
    type Iter: Iterator<Item = ViewBox>;
    fn into_view_iter(self) -> Self::Iter;
}

impl<T> IntoViewIter for T
where
    T: View + Send + Sync + 'static,
{
    type Iter = std::iter::Once<ViewBox>;
    fn into_view_iter(self) -> Self::Iter {
        std::iter::once(Box::new(self))
    }
}

// 2. 支持 Vec<T: View> (无论是 Vec<MyStruct> 还是 Vec<ViewBox>)
impl<T> IntoViewIter for Vec<T>
where
    T: View + Send + Sync + 'static,
{
    type Iter = std::iter::Map<std::vec::IntoIter<T>, fn(T) -> ViewBox>;
    fn into_view_iter(self) -> Self::Iter {
        self.into_iter().map(|v| Box::new(v) as ViewBox)
    }
}

impl IntoViewIter for ViewBox {
    type Iter = std::iter::Once<ViewBox>;
    fn into_view_iter(self) -> Self::Iter {
        std::iter::once(self)
    }
}

// 4. 处理 ViewBox 的 Vec (解决你报错的关键：div(vec![ViewBox, ViewBox]))
impl IntoViewIter for Vec<ViewBox> {
    type Iter = std::vec::IntoIter<ViewBox>;
    fn into_view_iter(self) -> Self::Iter {
        self.into_iter()
    }
}
