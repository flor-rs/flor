mod layout;
mod shared;
mod unit;

pub use {layout::*, shared::*, unit::*};

use crate::view::control_state::ControlState;
use crate::view::view_id::ViewId;
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
            unit_resolver: self.unit_resolver.clone(),
            current_key: self.current_key,
            state_variants: self.state_variants.clone(),
            cache_data: RwLock::new(Default::default()), // 缓存不需要克隆，会重新计算
            has_update: RwLock::new(Self::default_has_update()),
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
    const DEFAULT_HAS_UPDATE: [ControlState; 5] = [
        ControlState::Normal,
        ControlState::Focus,
        ControlState::Active,
        ControlState::Disabled,
        ControlState::Hover,
    ];

    #[inline]
    pub fn default_has_update() -> FxHashSet<ControlState> {
        let mut has_update_set = FxHashSet::default();
        has_update_set.extend(Self::DEFAULT_HAS_UPDATE);
        has_update_set
    }

    #[inline]
    pub fn new_with_compute_func(view_id: ViewId, compute_func: F) -> Self {
        Self {
            unit_resolver: UnitResolver::new(view_id),
            current_key: Default::default(),
            state_variants: Default::default(),
            cache_data: RwLock::new(Default::default()),
            has_update: RwLock::new(Self::default_has_update()),
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
        self.has_update.write().insert(self.current_key);
        self
    }
    #[inline]
    pub fn clear_all(mut self) -> Self {
        self.state_variants.clear();
        self.cache_data.write().clear();
        *self.has_update.write() = Self::default_has_update();
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
        if state_key == ControlState::Normal {
            self.cache_data.write().clear();
        } else {
            self.cache_data.write().remove(&state_key);
        }
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
            self.unit_resolver.sync_unit();
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
