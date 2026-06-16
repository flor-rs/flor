use crate::view::resolver::{extract_bracket_value, Unit, UnitMetrics};
use crate::view::{ViewId, VIEW_STORAGE};
use crate::windows::WindowEntryVisit;
use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use taffy::{Dimension, LengthPercentage, LengthPercentageAuto};

#[derive(Debug, Default, Clone)]
pub struct UnitResolver {
    view_id: ViewId,
    is_sync: Arc<AtomicBool>,
    unit: Arc<ArcSwap<UnitMetrics>>,
}

impl UnitResolver {
    /// 从 StateSelector 创建 UnitResolver
    pub fn new(view_id: ViewId) -> Self {
        Self {
            view_id,
            is_sync: Arc::new(AtomicBool::new(false)),
            unit: Arc::new(ArcSwap::from_pointee(UnitMetrics::default())),
        }
    }

    pub fn sync_unit(&self) -> bool {
        // 使用 Acquire 语义确保后续读取能看到最新的内存状态
        if self.is_sync.load(Ordering::Acquire) {
            return false;
        }

        if let Some(window_id) = VIEW_STORAGE.window_ids.read().get(self.view_id) {
            if let Some(global_unit_swap) = window_id.entry().map(|e| e.unit.load_full()) {
                self.unit.store(global_unit_swap);

                // 使用 Release 语义确保之前的 store 操作对其他线程可见
                self.is_sync.store(true, Ordering::Release);
                return true;
            }
        }
        false
    }

    // ========================================================================
    // 基础换算方法
    // ========================================================================

    /// rem 转 px
    #[inline]
    pub fn rem_to_px(&self, rem: f32) -> f32 {
        rem * self.unit.load().rem_px.load(Ordering::Relaxed)
    }

    /// pt 转 px (使用垂直 DPI)
    ///
    /// 公式: 1pt = dpi_y / 72 px
    #[inline]
    pub fn pt_to_px(&self, pt: f32) -> f32 {
        pt * self.unit.load().dpi_y.load(Ordering::Relaxed) / 72.0
    }

    /// pt 转 px (使用水平 DPI)
    #[inline]
    pub fn pt_to_px_x(&self, pt: f32) -> f32 {
        pt * self.unit.load().dpi_x.load(Ordering::Relaxed) / 72.0
    }

    /// vh 转 px
    /// 1vh = viewport_height / 100
    #[inline]
    pub fn vh_to_px(&self, vh: f32) -> f32 {
        vh * self.unit.load().viewport_height.load(Ordering::Relaxed) / 100.0
    }

    /// vw 转 px
    /// 1vw = viewport_width / 100
    #[inline]
    pub fn vw_to_px(&self, vw: f32) -> f32 {
        vw * self.unit.load().viewport_width.load(Ordering::Relaxed) / 100.0
    }

    // ========================================================================
    // 单位后缀 → px 核心转换 (所有解析方法共用)
    // ========================================================================

    /// 解析带单位后缀的值，返回 px 值。
    ///
    /// 仅处理 px / rem / pt / vh / vw 后缀，不处理 %、纯数字、关键字。
    /// 新增单位时只需修改此方法。
    #[inline]
    fn parse_unit_px(&self, value: &str) -> Option<f32> {
        if let Some(v) = value.strip_suffix("px") {
            return v.parse::<f32>().ok();
        }
        if let Some(v) = value.strip_suffix("rem") {
            return v.parse::<f32>().ok().map(|n| self.rem_to_px(n));
        }
        if let Some(v) = value.strip_suffix("pt") {
            return v.parse::<f32>().ok().map(|n| self.pt_to_px(n));
        }
        if let Some(v) = value.strip_suffix("vh") {
            return v.parse::<f32>().ok().map(|n| self.vh_to_px(n));
        }
        if let Some(v) = value.strip_suffix("vw") {
            return v.parse::<f32>().ok().map(|n| self.vw_to_px(n));
        }
        value.parse::<f32>().ok().map(|n| self.rem_to_px(n * 0.25))
    }

    /// 解析百分比后缀 `%`，返回百分比数值
    #[inline]
    fn parse_percent(value: &str) -> Option<f32> {
        value.strip_suffix('%').and_then(|v| v.parse::<f32>().ok())
    }

    /// 解析分数表达式 (如 `1/2` → 50.0)，返回百分比数值
    #[inline]
    fn parse_fraction(value: &str) -> Option<f32> {
        if !value.contains('/') {
            return None;
        }
        let parts: Vec<&str> = value.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(num), Ok(den)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                if den != 0.0 {
                    return Some(num / den * 100.0);
                }
            }
        }
        None
    }

    // ========================================================================
    // 通用长度解析
    // ========================================================================

    /// 解析长度字符串，支持 px、rem、pt、vh、vw 单位
    ///
    /// - `"16px"` -> 16.0
    /// - `"1rem"` -> 16.0 (如果 rem_px = 16)
    /// - `"16"` -> rem_to_px(16 * 0.25) (与 Tailwind 间距规则一致)
    pub fn parse_length(&self, value: &str) -> Option<f32> {
        self.parse_unit_px(value)
    }

    // ========================================================================
    // 布局长度解析
    // ========================================================================

    pub fn parse_length_percentage_auto(&self, value: &str) -> Option<LengthPercentageAuto> {
        if value == "auto" {
            return Some(LengthPercentageAuto::Auto);
        }
        if value == "full" {
            return Some(LengthPercentageAuto::Percent(100.0));
        }
        if let Some(pct) = Self::parse_percent(value) {
            return Some(LengthPercentageAuto::Percent(pct));
        }
        if let Some(pct) = Self::parse_fraction(value) {
            return Some(LengthPercentageAuto::Percent(pct));
        }
        if let Some(px) = self.parse_unit_px(value) {
            return Some(LengthPercentageAuto::Length(px));
        }
        None
    }

    pub fn parse_dimension(&self, value: &str) -> Option<Dimension> {
        if value == "auto" || value == "fit" || value == "min" || value == "max" {
            return Some(Dimension::Auto);
        }
        if value == "full" {
            return Some(Dimension::Percent(100.0));
        }
        if let Some(pct) = Self::parse_percent(value) {
            return Some(Dimension::Percent(pct));
        }
        if let Some(pct) = Self::parse_fraction(value) {
            return Some(Dimension::Percent(pct));
        }
        if let Some(px) = self.parse_unit_px(value) {
            return Some(Dimension::Length(px));
        }
        None
    }

    pub fn parse_length_percentage(&self, value: &str) -> Option<LengthPercentage> {
        if value == "full" {
            return Some(LengthPercentage::Percent(100.0));
        }
        if let Some(pct) = Self::parse_percent(value) {
            return Some(LengthPercentage::Percent(pct));
        }
        if let Some(pct) = Self::parse_fraction(value) {
            return Some(LengthPercentage::Percent(pct));
        }
        if let Some(px) = self.parse_unit_px(value) {
            return Some(LengthPercentage::Length(px));
        }
        None
    }

    // ========================================================================
    // Bracket-aware 便捷解析 (bracket 值优先，否则直接解析)
    // ========================================================================

    /// 解析 class 后缀，支持 `[value]` 括号语法和直接值
    ///
    /// 例如: `"[10px]"` → 先提取 `"10px"` 再解析; `"4"` → 直接解析
    #[inline]
    pub fn resolve_lpa(&self, suffix: &str) -> Option<LengthPercentageAuto> {
        extract_bracket_value(suffix)
            .and_then(|v| self.parse_length_percentage_auto(v))
            .or_else(|| self.parse_length_percentage_auto(suffix))
    }

    #[inline]
    pub fn resolve_dim(&self, suffix: &str) -> Option<Dimension> {
        extract_bracket_value(suffix)
            .and_then(|v| self.parse_dimension(v))
            .or_else(|| self.parse_dimension(suffix))
    }

    #[inline]
    pub fn resolve_lp(&self, suffix: &str) -> Option<LengthPercentage> {
        extract_bracket_value(suffix)
            .and_then(|v| self.parse_length_percentage(v))
            .or_else(|| self.parse_length_percentage(suffix))
    }

    pub fn parse_unit(&self, length: f32, value: Unit) -> f32 {
        match value {
            Unit::Px => length,
            Unit::Pt => self.pt_to_px(length),
            Unit::Rem => self.rem_to_px(length),
            Unit::Vw => self.vw_to_px(length),
            Unit::Vh => self.vh_to_px(length),
        }
    }

    // ========================================================================
    // Tailwind 字体大小
    // ========================================================================

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
