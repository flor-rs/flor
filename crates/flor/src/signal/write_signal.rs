use std::marker::PhantomData;
use crate::signal::id::Id;
use crate::signal::write::Write;

/// 只写信号
#[derive(Copy, Clone)]
pub struct WriteSignal<T> {
    pub(crate) id: Id,
    pub(crate) _type: PhantomData<T>,
}

impl<T> Write<T> for WriteSignal<T> {
    fn id(&self) -> Id {
        self.id
    }
}
