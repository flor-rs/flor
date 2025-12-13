use crate::view::control_state::ControlState;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct StateSelector<K: Eq + Hash + Clone, V: Clone> {
    pub current_key: ControlState,
    pub styles: FxHashMap<ControlState, FxHashMap<K, V>>,
    pub dirty_style: FxHashSet<ControlState>,
}

impl<K: Eq + Hash + Clone, V: Clone> Default for StateSelector<K, V> {
    fn default() -> Self {
        Self {
            current_key: ControlState::Normal,
            styles: Default::default(),
            dirty_style: Default::default(),
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> StateSelector<K, V> {
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
    pub fn clear(mut self) -> Self {
        if let Some(styles) = self.styles.get_mut(&self.current_key) {
            styles.clear();
        }
        self.mark_dirty(self.current_key);
        self
    }
    #[inline]
    pub fn clear_all(mut self) -> Self {
        self.styles.clear();
        self.dirty_style.clear();
        self
    }

    pub fn push(&mut self, k: K, v: V) {
        self.mark_dirty(self.current_key);
        self.styles
            .entry(self.current_key)
            .or_default()
            .insert(k, v);
    }

    pub fn update(&mut self, state_key: ControlState, k: K, v: V) {
        self.mark_dirty(self.current_key);
        self.styles.entry(state_key).or_default().insert(k, v);
    }

    #[inline]
    pub fn clear_dirty(&mut self, state: ControlState) {
        self.dirty_style.insert(state);
    }

    #[inline]
    pub fn mark_dirty(&mut self, state: ControlState) {
        self.dirty_style.remove(&state);
    }

    #[inline]
    pub fn is_dirty(&self, state: ControlState) -> bool {
        !self.dirty_style.contains(&state)
    }

    pub fn get_style(&self, state: ControlState) -> Option<FxHashMap<K, V>> {
        let expend_map = self.styles.get(&state).cloned().clone();
        if state == ControlState::Normal {
            return expend_map;
        }
        let Some(mut base_map) = self.styles.get(&ControlState::Normal).cloned().clone() else {
            return expend_map;
        };
        if let Some(expend_map) = expend_map {
            for (k, v) in expend_map {
                base_map.insert(k, v);
            }
        }
        Some(base_map)
    }
}
