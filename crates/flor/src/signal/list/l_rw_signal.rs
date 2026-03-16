use crate::signal::{
    create_list_signal, Id, ListRead, ListWrite, ReadListSignal, Signal, WriteListSignal, RUNTIME,
};
use std::marker::PhantomData;

/// 读写列表信号
pub struct RwListSignal<T> {
    id: Id,
    _type: PhantomData<T>,
}

impl<T> Clone for RwListSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RwListSignal<T> {}

impl<T: 'static> Default for RwListSignal<T> {
    fn default() -> Self {
        create_list_signal(Vec::<T>::new())
    }
}

impl<T> RwListSignal<T> {
    #[inline]
    pub fn new(id: Id) -> Self {
        Self {
            id,
            _type: Default::default(),
        }
    }

    pub fn split(self) -> (ReadListSignal<T>, WriteListSignal<T>) {
        (ReadListSignal::new(self.id), WriteListSignal::new(self.id))
    }
    pub fn as_read(&self) -> ReadListSignal<T> {
        ReadListSignal::new(self.id)
    }
    pub fn as_write(&self) -> WriteListSignal<T> {
        WriteListSignal::new(self.id)
    }

    #[allow(unused_variables)]
    pub fn set_label(&self, label: &str) {
        #[cfg(any(debug_assertions, feature = "signal-tracing"))]
        RUNTIME.labels.insert(self.id, label.into());
    }
}

impl<T> Signal for RwListSignal<T> {
    #[inline]
    fn id(&self) -> Id {
        self.id
    }
    #[inline]
    fn exists(&self) -> bool {
        RUNTIME.list_signal.contains_key(&self.id)
    }
}

impl<T> ListRead<T> for RwListSignal<T> {}

impl<T> ListWrite<T> for RwListSignal<T> {}
