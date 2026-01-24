// ============================================================================
// UnitResolver - 单位解析器
// ============================================================================

use atomic_float::AtomicF32;
use std::ops::Deref;
use std::sync::atomic::Ordering;
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
#[derive(Debug, Default)]
pub struct Unit {
    /// 1rem 对应的像素值 (默认 16.0)
    pub rem_px: AtomicF32,
    /// 水平方向 DPI (默认 96.0)
    pub dpi_x: AtomicF32,
    /// 垂直方向 DPI (默认 96.0)
    pub dpi_y: AtomicF32,
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

#[derive(Debug, Default)]
pub struct UnitResolver(Arc<Unit>);

impl UnitResolver {
    /// 从 StateSelector 创建 UnitResolver
    pub fn new(unit: Arc<Unit>) -> Self {
        Self(unit)
    }

    // ========================================================================
    // 基础换算方法
    // ========================================================================

    /// rem 转 px
    #[inline]
    pub fn rem_to_px(&self, rem: f32) -> f32 {
        rem * self.rem_px.load(Ordering::Relaxed)
    }

    /// pt 转 px (使用垂直 DPI)
    ///
    /// 公式: 1pt = dpi_y / 72 px
    #[inline]
    pub fn pt_to_px(&self, pt: f32) -> f32 {
        pt * self.dpi_y.load(Ordering::Relaxed) / 72.0
    }

    /// pt 转 px (使用水平 DPI)
    #[inline]
    pub fn pt_to_px_x(&self, pt: f32) -> f32 {
        pt * self.dpi_x.load(Ordering::Relaxed) / 72.0
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

    pub fn set_unit(&mut self, unit: Arc<Unit>) {
        self.0 = unit;
    }
}

impl Deref for UnitResolver {
    type Target = Unit;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
