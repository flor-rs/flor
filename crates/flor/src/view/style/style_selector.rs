use crate::view::control_state::ControlState;
use rustc_hash::FxHashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct StateSelector<K: Eq + Hash + Clone, V: Clone> {
    pub current_key: ControlState,
    pub styles: FxHashMap<ControlState, FxHashMap<K, V>>,
}

impl<K: Eq + Hash + Clone, V: Clone> Default for StateSelector<K, V> {
    fn default() -> Self {
        Self {
            current_key: ControlState::Normal,
            styles: Default::default(),
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
        self
    }
    #[inline]
    pub fn clear_all(mut self) -> Self {
        self.styles.clear();
        self
    }

    pub fn push(&mut self, k: K, v: V) {
        self.styles
            .entry(self.current_key)
            .or_default()
            .insert(k, v);
    }

    pub fn update(&mut self, state_key: ControlState, k: K, v: V) {
        self.styles.entry(state_key).or_default().insert(k, v);
    }
}
