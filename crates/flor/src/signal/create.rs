use crate::signal::{
    Id, ListItem, ReadListSignal, ReadSignal, RwListSignal, RwSignal, Value, WriteListSignal,
    WriteSignal, RUNTIME,
};

pub fn create_signal<T: 'static>(value: T) -> RwSignal<T> {
    let id = Id::next_signal_id();
    let signal = RwSignal::new(id);
    RUNTIME.values.insert(id, Value::new(value));
    signal
}

pub fn create_rw_signal<T: 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let id = Id::next_signal_id();
    let signal = RwSignal::new(id);
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

pub fn create_list_signal<T: 'static>(value: Vec<T>) -> RwListSignal<T> {
    let id = Id::next_list_id();
    let signal = RwListSignal::new(id);
    RUNTIME.list_signal.insert(
        id,
        value
            .into_iter()
            .map(|x| ListItem {
                id: Id::next_signal_id(),
                value: Value::new(x),
            })
            .collect(),
    );
    signal
}

pub fn create_rw_list_signal<T: 'static>(value: Vec<T>) -> (ReadListSignal<T>, WriteListSignal<T>) {
    create_list_signal(value).split()
}

#[allow(unused_variables)]
pub fn create_list_signal_with_label<T: 'static>(value: Vec<T>, label: &str) -> RwListSignal<T> {
    let rw_list = create_list_signal(value);
    #[cfg(any(debug_assertions, feature = "signal-tracing"))]
    rw_list.set_label(label);
    rw_list
}

pub fn create_rw_list_signal_with_label<T: 'static>(
    value: Vec<T>,
    label: &str,
) -> (ReadListSignal<T>, WriteListSignal<T>) {
    create_list_signal_with_label(value, label).split()
}
