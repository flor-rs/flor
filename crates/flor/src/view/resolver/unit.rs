// ============================================================================
// UnitResolver - 单位解析器
// ============================================================================

use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::entry::WindowEntryVisit;
use arc_swap::ArcSwap;
use atomic_float::AtomicF32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
#[derive(Debug)]
pub struct Unit {
    /// 1rem 对应的像素值 (默认 16.0)
    pub rem_px: AtomicF32,
    /// 水平方向 DPI (默认 96.0)
    pub dpi_x: AtomicF32,
    /// 垂直方向 DPI (默认 96.0)
    pub dpi_y: AtomicF32,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            rem_px: AtomicF32::new(16.),
            dpi_x: AtomicF32::new(96.),
            dpi_y: AtomicF32::new(96.),
        }
    }
}

impl Unit {
    pub fn new(dpi_x: f32, dpi_y: f32, rem_px: f32) -> Unit {
        Self {
            rem_px: AtomicF32::new(rem_px),
            dpi_x: AtomicF32::new(dpi_x),
            dpi_y: AtomicF32::new(dpi_y),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct UnitResolver {
    view_id: ViewId,
    is_sync: Arc<AtomicBool>,
    unit: Arc<ArcSwap<Unit>>,
}

impl UnitResolver {
    /// 从 StateSelector 创建 UnitResolver
    pub fn new(view_id: ViewId) -> Self {
        Self {
            view_id,
            is_sync: Arc::new(AtomicBool::new(false)),
            unit: Arc::new(ArcSwap::from_pointee(Unit::default())),
        }
    }

    pub fn sync_unit(&self) {
        // 使用 Acquire 语义确保后续读取能看到最新的内存状态
        if self.is_sync.load(Ordering::Acquire) {
            return;
        }

        if let Some(window_id) = VIEW_STORAGE.window_ids.read().get(self.view_id) {
            if let Some(global_unit_swap) = window_id.entry().map(|e| e.unit.load_full()) {
                self.unit.store(global_unit_swap);

                // 使用 Release 语义确保之前的 store 操作对其他线程可见
                self.is_sync.store(true, Ordering::Release);
            }
        }
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
