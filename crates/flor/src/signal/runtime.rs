use crate::signal::batch::BATCH;
use crate::signal::effect::signal_effect::SignalEffect;
use crate::signal::id::Id;
use crate::signal::value::Value;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rustc_hash::FxHashSet;
use std::cell::Cell;

pub static RUNTIME: Lazy<Runtime> = Lazy::new(Runtime::default);

thread_local! {
    /// RefCell<current_effect_id>
    pub static SCOPE: Cell<Option<Id>> = const { Cell::new(None) };
}

#[derive(Default)]
pub struct Runtime {
    // id,effects
    subscribe: DashMap<Id, FxHashSet<Id>>,
    pub(crate) values: DashMap<Id, Value>,
    pub(crate) effects: DashMap<Id, Box<dyn SignalEffect + Send + Sync>>,
    #[cfg(any(debug_assertions, feature = "signal-tracing"))]
    pub(crate) labels: DashMap<Id, String>,
    pub(crate) update_queue: Mutex<Vec<Id>>,
}

impl Runtime {
    pub fn subscribe(&self, signal_id: Id, scope_id: Id) {
        if let Some(mut val) = RUNTIME.subscribe.get_mut(&signal_id) {
            val.insert(scope_id);
        } else {
            let mut hash_set = FxHashSet::default();
            hash_set.insert(scope_id);
            RUNTIME.subscribe.insert(signal_id, hash_set);
        }
    }

    /// 运行指定信号订阅的副作用
    pub fn run_signal_effect(&self, signal_id: Id) {
        let batch_mode = BATCH.with(|b| b.borrow().batch_mode);
        if !batch_mode {
            self.insert_update_queue([signal_id]);
        } else {
            BATCH.with(|b| b.borrow_mut().signal_effect_ids.insert(signal_id));
        }
    }

    #[inline]
    pub(crate) fn insert_update_queue(&self, signal_ids: impl IntoIterator<Item=Id>) {
        self.update_queue.lock().extend(signal_ids);
    }

    /// 执行指定 signal_id 对应的所有 effect
    #[inline]
    pub(crate) fn run_effects_for_signal(&self, signal_id: Id) {
        let effect_ids = if let Some(subscribes) = self.subscribe.get(&signal_id) {
            subscribes.value().iter().copied().collect::<Vec<_>>()
        } else {
            vec![signal_id]
        };
        for effect_id in effect_ids {
            if let Some(effect) = self.effects.get(&effect_id) {
                effect.run_effect();
            }
        }
    }

    pub fn destroy_signal(&self, signal_id: Id) {
        self.values.remove(&signal_id);
        self.effects.remove(&signal_id);
        self.subscribe.remove(&signal_id);
        #[cfg(any(debug_assertions, feature = "signal-tracing"))]
        self.labels.remove(&signal_id);
    }

    pub(crate) fn execute_update_queue(&self) {
        while let Some(id) = self.update_queue.lock().pop() {
            self.run_effects_for_signal(id);
        }
    }
}
