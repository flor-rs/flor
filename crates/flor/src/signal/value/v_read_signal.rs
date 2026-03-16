use crate::signal::{Id, Read, Signal, RUNTIME};
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

impl<T> ReadSignal<T> {
    #[inline]
    pub fn new(id: Id) -> Self {
        Self {
            id,
            _type: Default::default(),
        }
    }
}

impl<T> Signal for ReadSignal<T> {
    #[inline]
    fn id(&self) -> Id {
        self.id
    }
    #[inline]
    fn exists(&self) -> bool {
        RUNTIME.values.contains_key(&self.id)
    }
}

impl<T> Read<T> for ReadSignal<T> {}
