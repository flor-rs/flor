use crate::signal::{Id, ListWrite, Signal, RUNTIME};
use std::marker::PhantomData;

/// 只写列表信号
pub struct WriteListSignal<T> {
    pub(crate) id: Id,
    pub(crate) _type: PhantomData<T>,
}

impl<T> Clone for WriteListSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WriteListSignal<T> {}

impl<T> WriteListSignal<T> {
    #[inline]
    pub fn new(id: Id) -> Self {
        Self {
            id,
            _type: Default::default(),
        }
    }
}

impl<T> Signal for WriteListSignal<T> {
    #[inline]
    fn id(&self) -> Id {
        self.id
    }
    #[inline]
    fn exists(&self) -> bool {
        RUNTIME.list_signal.contains_key(&self.id)
    }
}

impl<T> ListWrite<T> for WriteListSignal<T> {}
