mod layout;
mod shared;
mod unit;
mod decoration;

pub use {layout::*, shared::*, unit::*};

use crate::view::control_state::ControlState;
use parking_lot::{MappedRwLockReadGuard, RawRwLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug)]
pub struct Resolver<K, V, D, F>
where
    K: Eq + Hash + Clone,
    V: Clone,
    D: Clone,
    F: for<'a> Fn(&UnitResolver, ControlState, &FxHashMap<ControlState, FxHashMap<K, V>>) -> D,
{
    pub unit_resolver: UnitResolver,
    pub current_key: ControlState,
    pub state_variants: FxHashMap<ControlState, FxHashMap<K, V>>,
    pub cache_data: RwLock<FxHashMap<ControlState, D>>,
    pub has_update: RwLock<FxHashSet<ControlState>>,
    pub compute_func: Arc<F>,
}

impl<K, V, D, F> Clone for Resolver<K, V, D, F>
where
    K: Eq + Hash + Clone,
    V: Clone,
    D: Clone,
    F: for<'a> Fn(&UnitResolver, ControlState, &FxHashMap<ControlState, FxHashMap<K, V>>) -> D,
{
    fn clone(&self) -> Self {
        Self {
            unit_resolver: Default::default(),
            current_key: Default::default(),
            state_variants: Default::default(),
            cache_data: RwLock::new(Default::default()),
            has_update: Self::default_has_update(),
            compute_func: self.compute_func.clone(),
        }
    }
}

impl<K, V, D, F> Resolver<K, V, D, F>
where
    K: Eq + Hash + Clone,
    V: Clone,
    D: Clone,
    F: for<'a> Fn(&UnitResolver, ControlState, &FxHashMap<ControlState, FxHashMap<K, V>>) -> D,
{
    fn default_has_update() -> RwLock<FxHashSet<ControlState>> {
        let mut has_update = FxHashSet::default();
        has_update.insert(ControlState::Normal);
        has_update.insert(ControlState::Focus);
        has_update.insert(ControlState::Active);
        has_update.insert(ControlState::Disabled);
        has_update.insert(ControlState::Hover);
        RwLock::new(has_update)
    }

    #[inline]
    pub fn new_with_compute_func(compute_func: F) -> Self {
        Self {
            unit_resolver: Default::default(),
            current_key: Default::default(),
            state_variants: Default::default(),
            cache_data: RwLock::new(Default::default()),
            has_update: Self::default_has_update(),
            compute_func: Arc::new(compute_func),
        }
    }

    #[inline(always)]
    pub fn base(self) -> Self {
        self.normal()
    }
    #[inline]
    pub fn normal(mut self) -> Self {
        self.current_key = ControlState::Normal;
        self
    }
    #[inline]
    pub fn focus(mut self) -> Self {
        self.current_key = ControlState::Focus;
        self
    }
    #[inline]
    pub fn hover(mut self) -> Self {
        self.current_key = ControlState::Hover;
        self
    }
    #[inline]
    pub fn active(mut self) -> Self {
        self.current_key = ControlState::Active;
        self
    }
    #[inline]
    pub fn disabled(mut self) -> Self {
        self.current_key = ControlState::Disabled;
        self
    }
    pub fn clear(mut self) -> Self {
        if let Some(state_variants) = self.state_variants.get_mut(&self.current_key) {
            state_variants.clear();
        }
        self.cache_data.write().remove(&self.current_key);
        self
    }
    #[inline]
    pub fn clear_all(mut self) -> Self {
        self.state_variants.clear();
        self.cache_data.write().clear();
        self
    }

    pub fn push(&mut self, k: K, v: V) {
        self.state_variants
            .entry(self.current_key)
            .or_default()
            .insert(k, v);
        self.cache_data.write().remove(&self.current_key);
    }

    pub fn update(&mut self, state_key: ControlState, k: K, v: V) {
        self.state_variants
            .entry(state_key)
            .or_default()
            .insert(k, v);
        self.cache_data.write().remove(&self.current_key);
    }

    /// 0 拷贝 + 返回带锁引用 + 不可到达语义
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
            let data =
                (self.compute_func)(&self.unit_resolver, self.current_key, &self.state_variants);
            guard.insert(state, data);
            self.has_update.write().insert(state);
        }
        RwLockWriteGuard::downgrade(guard)
    }

    pub fn get_update_data_clone(&self, state: ControlState) -> Option<D> {
        if self.has_update.read().contains(&state) {
            let data = self.get_data_clone(state);
            self.has_update.write().remove(&state);
            Some(data)
        } else {
            None
        }
    }
}
