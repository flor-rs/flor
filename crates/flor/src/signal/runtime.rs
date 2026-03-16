use crate::signal::{Id, ListItem, SignalEffect, Value, BATCH};
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
    // id, Set<EffectId>
    subscribe: DashMap<Id, FxHashSet<Id>>,
    pub(crate) values: DashMap<Id, Value>,
    // EffectId，SignalEffect
    pub(crate) effects: DashMap<Id, Box<dyn SignalEffect + Send + Sync>>,
    pub(crate) effect_subscriptions: DashMap<Id, u32>,
    #[cfg(any(debug_assertions, feature = "signal-tracing"))]
    pub(crate) labels: DashMap<Id, String>,
    pub(crate) update_queue: Mutex<Vec<Id>>,
    pub(crate) list_signal: DashMap<Id, Vec<ListItem>>,
}

impl Runtime {
    pub fn subscribe(&self, signal_id: Id, effect_id: Id) {
        let insert = if let Some(mut val) = self.subscribe.get_mut(&signal_id) {
            val.insert(effect_id)
        } else {
            let mut hash_set = FxHashSet::default();
            hash_set.insert(effect_id);
            self.subscribe.insert(signal_id, hash_set);
            true
        };
        if insert {
            if let Some(mut val) = self.effect_subscriptions.get_mut(&effect_id) {
                *val += 1;
            } else {
                self.effect_subscriptions.insert(effect_id, 1);
            }
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
    pub(crate) fn insert_update_queue(&self, signal_ids: impl IntoIterator<Item = Id>) {
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
        // 如果是列表信号，先移除并递归销毁其行级信号 id
        if let Some((_, items)) = self.list_signal.remove(&signal_id) {
            for item in items {
                self.destroy_signal(item.id);
            }
        }

        self.values.remove(&signal_id);
        if let Some((_, effect_ids)) = self.subscribe.remove(&signal_id) {
            for effect_id in effect_ids {
                if let Some(mut subscriptions) = self.effect_subscriptions.get_mut(&effect_id) {
                    *subscriptions = subscriptions.saturating_sub(1);
                    if *subscriptions == 0 {
                        drop(subscriptions);
                        self.effect_subscriptions.remove(&effect_id);
                        self.effects.remove(&effect_id);
                        self.values.remove(&effect_id);
                    }
                }
            }
        }
        #[cfg(any(debug_assertions, feature = "signal-tracing"))]
        self.labels.remove(&signal_id);
    }

    pub(crate) fn execute_update_queue(&self) {
        loop {
            let id = self.update_queue.lock().pop();
            if let Some(id) = id {
                self.run_effects_for_signal(id);
            } else {
                break;
            }
        }
    }
}
