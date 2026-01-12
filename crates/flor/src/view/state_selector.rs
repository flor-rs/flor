mod decoration;
mod layout;

pub use decoration::*;
pub use layout::*;

use crate::view::control_state::ControlState;
use atomic_float::{AtomicF32, AtomicF64};
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StateSelector<K: Eq + Hash + Clone, V: Clone> {
    pub dpi_x: Arc<AtomicF64>,
    pub dpi_y: Arc<AtomicF64>,
    pub rem_px: Arc<AtomicF32>,
    pub current_key: ControlState,
    pub styles: FxHashMap<ControlState, FxHashMap<K, V>>,
    pub dirty_style: FxHashSet<ControlState>,
}

impl<K: Eq + Hash + Clone, V: Clone> Default for StateSelector<K, V> {
    fn default() -> Self {
        Self {
            dpi_x: Arc::new(AtomicF64::new(96.)),
            dpi_y: Arc::new(AtomicF64::new(96.)),
            rem_px: Arc::new(AtomicF32::new(16.)),
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
    #[inline]
    pub fn disabled(mut self) -> Self {
        self.current_key = ControlState::Disabled;
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

    pub fn set_dpi(&mut self, dpi_x: f64, dpi_y: f64) {
        self.dpi_x.store(dpi_x, Ordering::Release);
        self.dpi_y.store(dpi_y, Ordering::Release);
    }

    pub fn set_rem_px(&mut self, rem_px: f32) {
        self.rem_px.store(rem_px, Ordering::Release);
    }

    /// 获取单位解析器，用于长度换算
    pub fn unit_resolver(&self) -> UnitResolver {
        UnitResolver::from_selector(self)
    }
}

// ============================================================================
// UnitResolver - 单位解析器
// ============================================================================

/// 单位解析器
///
/// 封装 rem_px、dpi_x、dpi_y，提供便捷的长度换算方法。
///
/// # 示例
///
/// ```rust
/// let resolver = selector.unit_resolver();
/// let px_value = resolver.rem_to_px(1.5); // 1.5rem -> 24px (如果 rem_px = 16)
/// let pt_value = resolver.pt_to_px(12.0); // 12pt -> 16px (如果 dpi = 96)
/// ```
#[derive(Clone, Copy, Debug)]
pub struct UnitResolver {
    /// 1rem 对应的像素值 (默认 16.0)
    pub rem_px: f32,
    /// 水平方向 DPI (默认 96.0)
    pub dpi_x: f32,
    /// 垂直方向 DPI (默认 96.0)
    pub dpi_y: f32,
}

impl Default for UnitResolver {
    fn default() -> Self {
        Self {
            rem_px: 16.0,
            dpi_x: 96.0,
            dpi_y: 96.0,
        }
    }
}

impl UnitResolver {
    /// 从 StateSelector 创建 UnitResolver
    pub fn from_selector<K: Eq + Hash + Clone, V: Clone>(selector: &StateSelector<K, V>) -> Self {
        Self {
            rem_px: selector.rem_px.load(Ordering::Acquire),
            dpi_x: selector.dpi_x.load(Ordering::Acquire) as f32,
            dpi_y: selector.dpi_y.load(Ordering::Acquire) as f32,
        }
    }

    // ========================================================================
    // 基础换算方法
    // ========================================================================

    /// rem 转 px
    #[inline]
    pub fn rem_to_px(&self, rem: f32) -> f32 {
        rem * self.rem_px
    }

    /// pt 转 px (使用垂直 DPI)
    ///
    /// 公式: 1pt = dpi_y / 72 px
    #[inline]
    pub fn pt_to_px(&self, pt: f32) -> f32 {
        pt * self.dpi_y / 72.0
    }

    /// pt 转 px (使用水平 DPI)
    #[inline]
    pub fn pt_to_px_x(&self, pt: f32) -> f32 {
        pt * self.dpi_x / 72.0
    }

    // ========================================================================
    // 通用长度解析
    // ========================================================================

    /// 解析长度字符串，支持 px、rem、pt 单位
    ///
    /// # 示例
    ///
    /// - `"16px"` -> 16.0
    /// - `"1rem"` -> 16.0 (如果 rem_px = 16)
    /// - `"12pt"` -> 16.0 (如果 dpi = 96)
    /// - `"16"` -> 16.0 (纯数字)
    pub fn parse_length(&self, value: &str) -> Option<f32> {
        if let Some(v) = value.strip_suffix("px") {
            return v.parse::<f32>().ok();
        }
        if let Some(v) = value.strip_suffix("rem") {
            return v.parse::<f32>().ok().map(|n| self.rem_to_px(n));
        }
        if let Some(v) = value.strip_suffix("pt") {
            return v.parse::<f32>().ok().map(|n| self.pt_to_px(n));
        }
        value.parse::<f32>().ok()
    }

    /// 解析 Tailwind 字体大小，返回像素值
    ///
    /// 支持: xs, sm, base, lg, xl, 2xl, 3xl, 4xl, 5xl, 6xl, 7xl, 8xl, 9xl
    pub fn parse_tw_font_size(&self, name: &str) -> Option<f32> {
        let rem = match name {
            "xs" => 0.75,
            "sm" => 0.875,
            "base" => 1.0,
            "lg" => 1.125,
            "xl" => 1.25,
            "2xl" => 1.5,
            "3xl" => 1.875,
            "4xl" => 2.25,
            "5xl" => 3.0,
            "6xl" => 3.75,
            "7xl" => 4.5,
            "8xl" => 6.0,
            "9xl" => 8.0,
            _ => return None,
        };
        Some(self.rem_to_px(rem))
    }
}
