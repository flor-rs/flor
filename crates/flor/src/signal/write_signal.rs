use crate::signal::id::Id;
use crate::signal::write::Write;
use std::marker::PhantomData;

/// 只写信号
pub struct WriteSignal<T> {
    pub(crate) id: Id,
    pub(crate) _type: PhantomData<T>,
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WriteSignal<T> {}

impl<T> Write<T> for WriteSignal<T> {
    fn id(&self) -> Id {
        self.id
    }
}
