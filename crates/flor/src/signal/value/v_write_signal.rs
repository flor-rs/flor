use crate::signal::{Id, Signal, Write, RUNTIME};
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

impl<T> WriteSignal<T> {
    #[inline]
    pub fn new(id: Id) -> Self {
        Self {
            id,
            _type: Default::default(),
        }
    }
}

impl<T> Signal for WriteSignal<T> {
    #[inline]
    fn id(&self) -> Id {
        self.id
    }
    #[inline]
    fn exists(&self) -> bool {
        RUNTIME.values.contains_key(&self.id)
    }
}

impl<T> Write<T> for WriteSignal<T> {}
