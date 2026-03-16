use crate::signal::{Id, ListRead, Signal, RUNTIME};
use std::marker::PhantomData;

/// 只读列表信号
pub struct ReadListSignal<T> {
    pub(crate) id: Id,
    pub(crate) _type: PhantomData<T>,
}

impl<T> Clone for ReadListSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReadListSignal<T> {}

impl<T> ReadListSignal<T> {
    #[inline]
    pub fn new(id: Id) -> Self {
        Self {
            id,
            _type: Default::default(),
        }
    }
}

impl<T> Signal for ReadListSignal<T> {
    #[inline]
    fn id(&self) -> Id {
        self.id
    }
    #[inline]
    fn exists(&self) -> bool {
        RUNTIME.list_signal.contains_key(&self.id)
    }
}

impl<T> ListRead<T> for ReadListSignal<T> {}
