use crate::signal::{Id, ListItem, Signal, Value, RUNTIME, SCOPE};

pub trait ListWrite<T>: Signal {
    /// 跟踪
    fn track(&self) {
        if let Some(scope_id) = SCOPE.get() {
            RUNTIME.subscribe(self.id(), scope_id);
        }
    }

    fn push(&self, value: T)
    where
        T: 'static,
    {
        self.track();
        let mut guard = RUNTIME
            .list_signal
            .get_mut(&self.id())
            .expect("invalid list signal id: this signal has likely been destroyed");
        let item = ListItem {
            id: Id::next_signal_id(),
            value: Value::new(value),
        };
        let item_id = item.id;
        guard.value_mut().push(item);
        // 长度变化，同时通知新元素的订阅
        RUNTIME.run_signal_effect(self.id());
        RUNTIME.run_signal_effect(item_id);
    }

    fn try_push(&self, value: T) -> bool
    where
        T: 'static,
    {
        self.track();
        if let Some(mut guard) = RUNTIME.list_signal.get_mut(&self.id()) {
            let item = ListItem {
                id: Id::next_signal_id(),
                value: Value::new(value),
            };
            let item_id = item.id;
            guard.value_mut().push(item);
            RUNTIME.run_signal_effect(self.id());
            RUNTIME.run_signal_effect(item_id);
            true
        } else {
            false
        }
    }

    fn set(&self, index: usize, value: T)
    where
        T: 'static,
    {
        self.track();
        let mut guard = RUNTIME
            .list_signal
            .get_mut(&self.id())
            .expect("invalid list signal id: this signal has likely been destroyed");
        let vec = guard.value_mut();
        if let Some(slot) = vec.get_mut(index) {
            slot.value = Value::new(value);
            RUNTIME.run_signal_effect(slot.id);
        } else {
            panic!("list index out of bounds");
        }
    }

    fn try_set(&self, index: usize, value: T) -> bool
    where
        T: 'static,
    {
        self.track();
        if let Some(mut guard) = RUNTIME.list_signal.get_mut(&self.id()) {
            let vec = guard.value_mut();
            if let Some(slot) = vec.get_mut(index) {
                slot.value = Value::new(value);
                RUNTIME.run_signal_effect(slot.id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn insert(&self, index: usize, value: T)
    where
        T: 'static,
    {
        self.track();
        let mut guard = RUNTIME
            .list_signal
            .get_mut(&self.id())
            .expect("invalid list signal id: this signal has likely been destroyed");
        let vec = guard.value_mut();
        if index > vec.len() {
            panic!("list index out of bounds");
        }
        let item = ListItem {
            id: Id::next_signal_id(),
            value: Value::new(value),
        };
        vec.insert(index, item);
        RUNTIME.run_signal_effect(self.id());
        RUNTIME.run_signal_effect(vec[index].id);
    }

    fn try_insert(&self, index: usize, value: T) -> bool
    where
        T: 'static,
    {
        self.track();
        if let Some(mut guard) = RUNTIME.list_signal.get_mut(&self.id()) {
            let vec = guard.value_mut();
            if index <= vec.len() {
                let item = ListItem {
                    id: Id::next_signal_id(),
                    value: Value::new(value),
                };
                vec.insert(index, item);
                RUNTIME.run_signal_effect(self.id());
                RUNTIME.run_signal_effect(vec[index].id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn remove(&self, index: usize) -> T
    where
        T: 'static,
    {
        self.track();
        let mut guard = RUNTIME
            .list_signal
            .get_mut(&self.id())
            .expect("invalid list signal id: this signal has likely been destroyed");
        let item = guard.value_mut().remove(index);
        let value = item
            .value
            .into_inner::<T>()
            .expect("to downcast list item type fail");
        RUNTIME.run_signal_effect(self.id());
        RUNTIME.run_signal_effect(item.id);
        value
    }

    fn try_remove(&self, index: usize) -> Option<T>
    where
        T: 'static,
    {
        self.track();
        if let Some(mut guard) = RUNTIME.list_signal.get_mut(&self.id()) {
            if index < guard.len() {
                let item = guard.value_mut().remove(index);
                let value = item
                    .value
                    .into_inner::<T>()
                    .expect("try_remove: to downcast list item type fail");
                RUNTIME.run_signal_effect(self.id());
                RUNTIME.run_signal_effect(item.id);
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn clear(&self) {
        self.track();
        let mut guard = RUNTIME
            .list_signal
            .get_mut(&self.id())
            .expect("invalid list signal id: this signal has likely been destroyed");
        let ids: Vec<Id> = guard.value().iter().map(|item| item.id).collect();
        guard.value_mut().clear();
        RUNTIME.run_signal_effect(self.id());
        for id in ids {
            RUNTIME.run_signal_effect(id);
        }
    }

    fn try_clear(&self) -> bool {
        self.track();
        if let Some(mut guard) = RUNTIME.list_signal.get_mut(&self.id()) {
            let ids: Vec<Id> = guard.value().iter().map(|item| item.id).collect();
            guard.value_mut().clear();
            RUNTIME.run_signal_effect(self.id());
            for id in ids {
                RUNTIME.run_signal_effect(id);
            }
            true
        } else {
            false
        }
    }

    fn update(&self, index: usize, f: impl FnOnce(&mut T))
    where
        T: 'static,
    {
        self.track();
        let mut guard = RUNTIME
            .list_signal
            .get_mut(&self.id())
            .expect("invalid list signal id: this signal has likely been destroyed");
        let vec = guard.value_mut();
        let slot = vec.get_mut(index).expect("list index out of bounds");
        let slot_value = slot
            .value
            .get_mut_ref::<T>()
            .expect("to downcast list item type fail");
        f(slot_value);
        RUNTIME.run_signal_effect(slot.id);
    }

    fn try_update(&self, index: usize, f: impl FnOnce(&mut T)) -> bool
    where
        T: 'static,
    {
        self.track();
        if let Some(mut guard) = RUNTIME.list_signal.get_mut(&self.id()) {
            let vec = guard.value_mut();
            if let Some(slot) = vec.get_mut(index) {
                let t = slot
                    .value
                    .get_mut_ref::<T>()
                    .expect("try_update: to downcast list item type fail");
                f(t);
                RUNTIME.run_signal_effect(slot.id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
