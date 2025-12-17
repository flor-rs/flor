use crate::signal::id::Id;
use crate::signal::read::Read;
use crate::signal::read_signal::ReadSignal;
use crate::signal::runtime::RUNTIME;
use crate::signal::value::Value;
use crate::signal::write::Write;
use crate::signal::write_signal::WriteSignal;
use std::marker::PhantomData;

/// 读写信号
#[derive(Copy, Clone)]
pub struct RwSignal<T> {
    id: Id,
    _type: PhantomData<T>,
}

impl<T: Default + 'static> Default for RwSignal<T> {
    fn default() -> Self {
        create_signal(T::default())
    }
}

impl<T> RwSignal<T> {
    pub fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (
            ReadSignal {
                id: self.id,
                _type: self._type,
            },
            WriteSignal {
                id: self.id,
                _type: self._type,
            },
        )
    }
    pub fn as_read(&self) -> ReadSignal<T> {
        ReadSignal {
            id: self.id,
            _type: self._type,
        }
    }
    pub fn as_write(&self) -> WriteSignal<T> {
        WriteSignal {
            id: self.id,
            _type: self._type,
        }
    }

    #[allow(unused_variables)]
    pub fn set_label(&self, label: &str) {
        #[cfg(any(debug_assertions, feature = "signal-tracing"))]
        RUNTIME.labels.insert(self.id, label.into());
    }
}

impl<T> Read<T> for RwSignal<T> {
    #[inline]
    fn id(&self) -> Id {
        self.id
    }
}

impl<T> Write<T> for RwSignal<T> {
    #[inline]
    fn id(&self) -> Id {
        self.id
    }
}

pub fn create_signal<T: 'static>(value: T) -> RwSignal<T> {
    let id = Id::next();
    let signal = RwSignal::<T> {
        id,
        _type: PhantomData,
    };
    RUNTIME.values.insert(id, Value::new(value));
    signal
}

pub fn create_rw_signal<T: 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let id = Id::next();
    let signal = RwSignal::<T> {
        id,
        _type: PhantomData,
    };
    RUNTIME.values.insert(id, Value::new(value));
    signal.split()
}

#[allow(unused_variables)]
pub fn create_signal_with_label<T: 'static>(value: T, label: &str) -> RwSignal<T> {
    let rw_signal = create_signal(value);
    #[cfg(any(debug_assertions, feature = "signal-tracing"))]
    rw_signal.set_label(label);
    rw_signal
}

pub fn create_rw_signal_with_label<T: 'static>(
    value: T,
    label: &str,
) -> (ReadSignal<T>, WriteSignal<T>) {
    create_signal_with_label(value, label).split()
}
