mod layout;
mod shared;
mod unit;

pub use {layout::*, shared::*, unit::*};

use crate::view::control_state::ControlState;
use crate::view::ViewId;
use parking_lot::{MappedRwLockReadGuard, RawRwLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use rustc_hash::FxHashMap;
use slotmap::{new_key_type, SlotMap};
use small_map::FxSmallMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

new_key_type! {
    pub struct LayerId;
}

const RESOLVER_INLINE_LIMIT: usize = 24;

pub type ResolverComputeMap<K, V> = FxSmallMap<RESOLVER_INLINE_LIMIT, K, V>;
pub type ResolverVariantsMap<K, V> =
    FxSmallMap<RESOLVER_INLINE_LIMIT, K, FxSmallMap<4, ControlState, V>>;

#[derive(Debug)]
pub struct Resolver<K, V, D, F>
where
    K: Eq + Hash + Clone,
    V: Clone,
    D: Clone,
    F: for<'a> Fn(&UnitResolver, &ResolverComputeMap<K, V>) -> D,
{
    pub unit_resolver: UnitResolver,
    pub current_control_state: ControlState,
    pub current_layer_id: LayerId,
    pub layer_seq: Vec<LayerId>,
    pub state_layer: SlotMap<LayerId, ResolverVariantsMap<K, V>>,
    pub cache_data: RwLock<FxHashMap<ControlState, D>>,
    pub compute_func: Arc<F>,
    pub dirty: AtomicBool,
}

impl<K, V, D, F> Clone for Resolver<K, V, D, F>
where
    K: Eq + Hash + Clone,
    V: Clone,
    D: Clone,
    F: for<'a> Fn(&UnitResolver, &ResolverComputeMap<K, V>) -> D,
{
    fn clone(&self) -> Self {
        Self {
            unit_resolver: self.unit_resolver.clone(),
            current_control_state: self.current_control_state,
            current_layer_id: self.current_layer_id,
            layer_seq: self.layer_seq.clone(),
            state_layer: self.state_layer.clone(),
            cache_data: RwLock::new(Default::default()),
            compute_func: self.compute_func.clone(),
            dirty: AtomicBool::new(true),
        }
    }
}

impl<K, V, D, F> Resolver<K, V, D, F>
where
    K: Eq + Hash + Clone,
    V: Clone,
    D: Clone,
    F: for<'a> Fn(&UnitResolver, &ResolverComputeMap<K, V>) -> D,
{
    #[inline]
    pub fn new_with_compute_func(view_id: ViewId, compute_func: F) -> Self {
        let mut state_variants = SlotMap::with_key();
        let current_layer_id = state_variants.insert(Default::default());
        Self {
            unit_resolver: UnitResolver::new(view_id),
            current_control_state: Default::default(),
            current_layer_id,
            layer_seq: vec![current_layer_id],
            state_layer: state_variants,
            cache_data: RwLock::new(Default::default()),
            compute_func: Arc::new(compute_func),
            dirty: AtomicBool::new(true),
        }
    }

    pub fn new_layer(&mut self) -> LayerId {
        let layer_id = self.state_layer.insert(Default::default());
        self.layer_seq.push(layer_id);
        layer_id
    }

    pub fn switch_layer(&mut self, layer_id: LayerId) {
        self.current_layer_id = layer_id;
    }

    pub fn switch_control_state(&mut self, control_state: ControlState) {
        self.current_control_state = control_state;
    }

    pub fn clear_layer_variants(&mut self, layer_id: LayerId) {
        if let Some(map) = self.state_layer.get_mut(layer_id) {
            map.clear();
        }
        self.cache_data.write().clear();
    }

    #[inline(always)]
    pub fn base(self) -> Self {
        self.normal()
    }
    #[inline]
    pub fn normal(mut self) -> Self {
        self.current_control_state = ControlState::Normal;
        self
    }
    #[inline]
    pub fn focus(mut self) -> Self {
        self.current_control_state = ControlState::Focus;
        self
    }
    #[inline]
    pub fn hover(mut self) -> Self {
        self.current_control_state = ControlState::Hover;
        self
    }
    #[inline]
    pub fn active(mut self) -> Self {
        self.current_control_state = ControlState::Active;
        self
    }
    #[inline]
    pub fn disabled(mut self) -> Self {
        self.current_control_state = ControlState::Disabled;
        self
    }
    pub fn clear(mut self) -> Self {
        self.state_layer.clear();
        self.clear_cache();
        self
    }

    #[inline]
    pub fn clear_cache(&mut self) {
        self.cache_data.write().clear();
        self.dirty.store(true, Ordering::Release);
    }

    pub fn push(&mut self, k: K, v: V) {
        if let Some(state_variants) = self.state_layer.get_mut(self.current_layer_id) {
            if let Some(state_map) = state_variants.get_mut(&k) {
                state_map.insert(self.current_control_state, v);
            } else {
                state_variants.insert(k, FxSmallMap::from_iter([(self.current_control_state, v)]));
            }
            self.clear_cache();
        }
    }

    pub fn update(&mut self, state_key: ControlState, k: K, v: V) {
        if let Some(layer_map) = self.state_layer.get_mut(self.current_layer_id) {
            if let Some(state_map) = layer_map.get_mut(&k) {
                state_map.insert(state_key, v);
            } else {
                layer_map.insert(k, FxSmallMap::from_iter([(state_key, v)]));
            }

            self.clear_cache();
        }
    }

    pub fn get_data_borrow(&self, state: ControlState) -> MappedRwLockReadGuard<'_, D> {
        {
            let guard = self.cache_data.read();
            if guard.contains_key(&state) {
                return RwLockReadGuard::map(guard, |map| &map[&state]);
            }
        }

        let guard = self.compute(state);

        RwLockReadGuard::map(guard, |map| &map[&state])
    }

    pub fn get_data_clone(&self, state: ControlState) -> D {
        {
            let guard = self.cache_data.read();
            if guard.contains_key(&state) {
                return guard[&state].clone();
            }
        }
        let guard = self.compute(state);
        guard[&state].clone()
    }

    #[inline]
    fn compute(
        &self,
        state: ControlState,
    ) -> parking_lot::lock_api::RwLockReadGuard<'_, RawRwLock, FxHashMap<ControlState, D>> {
        let mut guard = self.cache_data.write();
        if !guard.contains_key(&state) {
            let just_synced = self.unit_resolver.sync_unit();
            if just_synced {
                guard.clear();
            }

            let mut merged_variants = FxSmallMap::<24, _, _>::new();
            for &layer_id in self.layer_seq.iter() {
                if let Some(state_variants) = self.state_layer.get(layer_id) {
                    for (k, v) in state_variants {
                        match state {
                            ControlState::Focus | ControlState::Hover => {
                                // 继承默认
                                if let Some(v) = v.get(&ControlState::Normal) {
                                    merged_variants.insert(k.clone(), v.clone());
                                }
                                if let Some(v) = v.get(&state) {
                                    merged_variants.insert(k.clone(), v.clone());
                                }
                            }
                            ControlState::Active => {
                                // 继承 focus
                                if let Some(v) = v.get(&ControlState::Normal) {
                                    merged_variants.insert(k.clone(), v.clone());
                                }
                                if let Some(v) = v.get(&ControlState::Focus) {
                                    merged_variants.insert(k.clone(), v.clone());
                                }
                                if let Some(v) = v.get(&state) {
                                    merged_variants.insert(k.clone(), v.clone());
                                }
                            }
                            ControlState::Normal | ControlState::Disabled => {
                                // 不继承
                                if let Some(v) = v.get(&state) {
                                    merged_variants.insert(k.clone(), v.clone());
                                }
                            }
                        }
                    }
                }
            }
            let data = (self.compute_func)(&self.unit_resolver, &merged_variants);

            guard.insert(state, data);
        }
        RwLockWriteGuard::downgrade(guard)
    }

    pub fn get_data_if_changed(&self, state: ControlState) -> Option<D> {
        let dirty = self.dirty.load(Ordering::Acquire);
        if dirty {
            // 脏了一定要去拿数据
            return Some(self.get_data_clone(state));
        }
        None
    }
}
