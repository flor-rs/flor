use crate::signal::id::Id;
use crate::signal::read::Read;
use std::marker::PhantomData;

/// 只读信号
pub struct ReadSignal<T> {
    pub(crate) id: Id,
    pub(crate) _type: PhantomData<T>,
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReadSignal<T> {}

impl<T> Read<T> for ReadSignal<T> {
    fn id(&self) -> Id {
        self.id
    }
}
