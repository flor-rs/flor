mod layout;
mod shared;
mod unit;

pub use {layout::*, shared::*, unit::*};

use crate::view::control_state::ControlState;
use crate::view::ViewId;
use parking_lot::{MappedRwLockReadGuard, RawRwLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use rustc_hash::FxHashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, Default)]
pub enum ResolverLayer {
    Default,
    #[default]
    Normal,
}

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
    #[cfg(feature = "class")]
    pub class_state_variants: FxHashMap<ControlState, FxHashMap<K, V>>,
    pub layer: ResolverLayer,
    pub default_state_variants: FxHashMap<ControlState, FxHashMap<K, V>>,
    pub state_variants: FxHashMap<ControlState, FxHashMap<K, V>>,
    pub cache_data: RwLock<FxHashMap<ControlState, D>>,
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
            #[cfg(feature = "class")]
            class_state_variants: self.class_state_variants.clone(),
            layer: ResolverLayer::default(),
            default_state_variants: self.default_state_variants.clone(),
            state_variants: self.state_variants.clone(),
            cache_data: RwLock::new(Default::default()), // 缓存不需要克隆，会重新计算
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
    #[inline]
    pub fn new_with_compute_func(view_id: ViewId, compute_func: F) -> Self {
        Self {
            unit_resolver: UnitResolver::new(view_id),
            current_key: Default::default(),
            #[cfg(feature = "class")]
            class_state_variants: Default::default(),
            layer: ResolverLayer::Normal,
            default_state_variants: Default::default(),
            state_variants: Default::default(),
            cache_data: RwLock::new(Default::default()),
            compute_func: Arc::new(compute_func),
        }
    }

    pub fn default_layer(mut self) -> Self {
        self.layer = ResolverLayer::Default;
        self
    }

    pub fn normal_layer(mut self) -> Self {
        self.layer = ResolverLayer::Normal;
        self
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
        match self.layer {
            ResolverLayer::Default => {
                if let Some(state_variants) = self.default_state_variants.get_mut(&self.current_key)
                {
                    state_variants.clear();
                }
            }
            ResolverLayer::Normal => {
                if let Some(state_variants) = self.state_variants.get_mut(&self.current_key) {
                    state_variants.clear();
                }
            }
        }
        self.cache_data.write().remove(&self.current_key);
        self
    }
    #[inline]
    pub fn clear_all(mut self) -> Self {
        self.default_state_variants.clear();
        #[cfg(feature = "class")]
        self.class_state_variants.clear();
        self.state_variants.clear();
        self.cache_data.write().clear();
        self
    }

    pub fn push(&mut self, k: K, v: V) {
        match self.layer {
            ResolverLayer::Default => {
                self.default_state_variants
                    .entry(self.current_key)
                    .or_default()
                    .insert(k, v);
            }
            ResolverLayer::Normal => {
                self.state_variants
                    .entry(self.current_key)
                    .or_default()
                    .insert(k, v);
            }
        }
        self.cache_data.write().remove(&self.current_key);
    }

    pub fn update(&mut self, state_key: ControlState, k: K, v: V) {
        match self.layer {
            ResolverLayer::Default => {
                self.default_state_variants
                    .entry(state_key)
                    .or_default()
                    .insert(k, v);
            }
            ResolverLayer::Normal => {
                self.state_variants
                    .entry(state_key)
                    .or_default()
                    .insert(k, v);
            }
        }
        if state_key == ControlState::Normal {
            self.cache_data.write().clear();
        } else {
            self.cache_data.write().remove(&state_key);
        }
    }

    #[cfg(feature = "class")]
    pub fn class_update(&mut self, state_key: ControlState, k: K, v: V) {
        self.class_state_variants
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
            let just_synced = self.unit_resolver.sync_unit();
            if just_synced {
                guard.clear();
            }

            let no_default = self.default_state_variants.is_empty();
            let no_state = self.state_variants.is_empty();

            #[cfg(feature = "class")]
            let no_class = self.class_state_variants.is_empty();

            let merge_map =
                |dst: &mut FxHashMap<ControlState, FxHashMap<K, V>>,
                 src: &FxHashMap<ControlState, FxHashMap<K, V>>| {
                    for (state_key, map) in src.iter() {
                        if let Some(existing_map) = dst.get_mut(state_key) {
                            existing_map.extend(map.iter().map(|(k, v)| (k.clone(), v.clone())));
                        } else {
                            dst.insert(*state_key, map.clone());
                        }
                    }
                };

            #[cfg(feature = "class")]
            let data = if no_default && no_class {
                (self.compute_func)(&self.unit_resolver, self.current_key, &self.state_variants)
            } else if no_default && no_state {
                (self.compute_func)(
                    &self.unit_resolver,
                    self.current_key,
                    &self.class_state_variants,
                )
            } else if no_class && no_state {
                (self.compute_func)(
                    &self.unit_resolver,
                    self.current_key,
                    &self.default_state_variants,
                )
            } else {
                let mut merged_variants = self.default_state_variants.clone();
                merge_map(&mut merged_variants, &self.class_state_variants);
                merge_map(&mut merged_variants, &self.state_variants);
                (self.compute_func)(&self.unit_resolver, self.current_key, &merged_variants)
            };

            #[cfg(not(feature = "class"))]
            let data = if no_default {
                (self.compute_func)(&self.unit_resolver, self.current_key, &self.state_variants)
            } else if no_state {
                (self.compute_func)(
                    &self.unit_resolver,
                    self.current_key,
                    &self.default_state_variants,
                )
            } else {
                let mut merged_variants = self.default_state_variants.clone();
                merge_map(&mut merged_variants, &self.state_variants);
                (self.compute_func)(&self.unit_resolver, self.current_key, &merged_variants)
            };

            guard.insert(state, data);
        }
        RwLockWriteGuard::downgrade(guard)
    }

    pub fn get_update_data_clone(&self, state: ControlState) -> Option<D> {
        if !self.cache_data.read().contains_key(&state) {
            let data = self.get_data_clone(state);
            Some(data)
        } else {
            None
        }
    }
}
