use crate::signal::{create_signal, Id, Read, ReadSignal, Signal, Write, WriteSignal, RUNTIME};
use std::marker::PhantomData;

/// 读写信号
pub struct RwSignal<T> {
    id: Id,
    _type: PhantomData<T>,
}

impl<T> Clone for RwSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RwSignal<T> {}

impl<T: Default + 'static> Default for RwSignal<T> {
    fn default() -> Self {
        create_signal(T::default())
    }
}

impl<T> RwSignal<T> {
    #[inline]
    pub fn new(id: Id) -> Self {
        Self {
            id,
            _type: Default::default(),
        }
    }

    pub fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (ReadSignal::new(self.id), WriteSignal::new(self.id))
    }
    pub fn as_read(&self) -> ReadSignal<T> {
        ReadSignal::new(self.id)
    }
    pub fn as_write(&self) -> WriteSignal<T> {
        WriteSignal::new(self.id)
    }

    #[allow(unused_variables)]
    pub fn set_label(&self, label: &str) {
        #[cfg(any(debug_assertions, feature = "signal-tracing"))]
        RUNTIME.labels.insert(self.id, label.into());
    }
}

impl<T> Signal for RwSignal<T> {
    #[inline]
    fn id(&self) -> Id {
        self.id
    }
    #[inline]
    fn exists(&self) -> bool {
        RUNTIME.values.contains_key(&self.id)
    }
}

impl<T> Read<T> for RwSignal<T> {}

impl<T> Write<T> for RwSignal<T> {}
