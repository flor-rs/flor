//! Decoration Parser - TW风格装饰语法解析器
//!
//! 这是一个轻量级、独立的解析器，不依赖 StateSelector。
//! 用户可以手动调用解析，获得结果后自行使用。
//!
//! # 使用示例
//!
//! ```rust
//! use flor::view::state_selector::DecorationParser;
//!
//! // 解析一组类名
//! let result = DecorationParser::parse(&["bg-white", "border", "border-gray-300", "rounded-lg"]);
//!
//! // 使用解析结果
//! if let Some(bg) = result.background_color {
//!     // 绘制背景...
//! }
//!
//! // 带状态前缀的解析
//! let result = DecorationParser::parse(&[
//!     "bg-white",
//!     "hover:bg-gray-100",
//!     "border",
//!     "hover:border-blue-500",
//! ]);
//! // result.normal - 基础样式
//! // result.hover  - hover 状态样式 (已合并基础样式)
//! ```
//!
//! # 支持的语法
//!
//! - Border: `border`, `border-0/2/4/8`, `border-t/b/l/r-*`, `border-{color}-{shade}`
//! - Rounded: `rounded`, `rounded-sm/md/lg/xl/2xl/3xl/full`, `rounded-t/b/l/r-*`, `rounded-[Npx]`
//! - Background: `bg-{color}-{shade}`, `bg-[#hex]`, `bg-transparent`
//! - Opacity: `opacity-0/50/100`, `opacity-[0.5]`
//!
//! 支持状态前缀: `hover:`, `focus:`, `active:`, `disabled:`

use super::shared::{extract_bracket_value, parse_color, parse_length, parse_state_prefix};
use crate::view::control_state::ControlState;
use flor_base::types::Color;

// ============================================================================
// Public Types
// ============================================================================

/// 边框矩形 (四边)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl BorderRect {
    #[inline]
    pub const fn all(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::all(0.0)
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.left == 0.0 && self.right == 0.0 && self.top == 0.0 && self.bottom == 0.0
    }
}

/// 圆角矩形 (四角)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadius {
    #[inline]
    pub const fn all(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::all(0.0)
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }
}

/// 单状态的装饰样式
#[derive(Clone, Debug, Default)]
pub struct DecorationStyle {
    /// 边框宽度 (left, right, top, bottom) - 单位: px
    pub border_width: BorderRect,
    /// 边框颜色
    pub border_color: Option<Color>,
    /// 圆角 (top-left, top-right, bottom-right, bottom-left) - 单位: px
    pub border_radius: CornerRadius,
    /// 背景颜色
    pub background_color: Option<Color>,
    /// 透明度 (0.0 - 1.0, None = 未设置)
    pub opacity: Option<f32>,
}

impl DecorationStyle {
    /// 创建空样式
    #[inline]
    pub const fn new() -> Self {
        Self {
            border_width: BorderRect::zero(),
            border_color: None,
            border_radius: CornerRadius::zero(),
            background_color: None,
            opacity: None,
        }
    }

    /// 合并另一个样式 (other 覆盖 self)
    pub fn merge(&mut self, other: &DecorationStyle) {
        // Border width: 如果 other 有设置，覆盖
        if !other.border_width.is_zero() {
            self.border_width = other.border_width;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        if !other.border_radius.is_zero() {
            self.border_radius = other.border_radius;
        }
        if other.background_color.is_some() {
            self.background_color = other.background_color;
        }
        if other.opacity.is_some() {
            self.opacity = other.opacity;
        }
    }

    /// 获取最终透明度 (默认 1.0)
    #[inline]
    pub fn get_opacity(&self) -> f32 {
        self.opacity.unwrap_or(1.0)
    }
}

/// 装饰样式 - 包含解析和结果
///
/// # 使用示例
///
/// ```rust
/// use flor::view::state_selector::{Decoration, DecorationState};
///
/// // 解析类名
/// let deco = Decoration::parse(&["bg-white", "border", "rounded-lg", "hover:bg-gray-100"]);
///
/// // 获取样式
/// let style = deco.get(DecorationState::Normal);
/// let hover_style = deco.get(DecorationState::Hover);
/// ```
#[derive(Clone, Debug, Default)]
pub struct Decoration {
    /// Normal 状态样式 (基础)
    pub normal: DecorationStyle,
    /// Hover 状态样式 (继承 normal 后覆盖)
    pub hover: DecorationStyle,
    /// Focus 状态样式 (继承 normal 后覆盖)
    pub focus: DecorationStyle,
    /// Active 状态样式 (继承 normal 后覆盖)
    pub active: DecorationStyle,
    /// Disabled 状态样式 (继承 normal 后覆盖)
    pub disabled: DecorationStyle,
}

impl Decoration {
    /// 创建空的装饰样式
    #[inline]
    pub const fn new() -> Self {
        Self {
            normal: DecorationStyle::new(),
            hover: DecorationStyle::new(),
            focus: DecorationStyle::new(),
            active: DecorationStyle::new(),
            disabled: DecorationStyle::new(),
        }
    }

    /// 解析类名列表
    ///
    /// # 参数
    /// - `classes`: 类名切片，如 `&["bg-white", "hover:bg-gray-100", "rounded-lg"]`
    ///
    /// # 示例
    /// ```rust
    /// let deco = Decoration::parse(&["bg-white", "border", "rounded-lg"]);
    /// ```
    pub fn parse(classes: &[&str]) -> Self {
        Self::parse_with_rem(classes, 16.0)
    }

    /// 解析类名列表，指定 rem 基准值
    ///
    /// # 参数
    /// - `classes`: 类名切片
    /// - `rem_px`: 1rem 等于多少 px (默认 16.0)
    pub fn parse_with_rem(classes: &[&str], rem_px: f32) -> Self {
        let mut normal_acc = StyleAccumulator::default();
        let mut hover_acc = StyleAccumulator::default();
        let mut focus_acc = StyleAccumulator::default();
        let mut active_acc = StyleAccumulator::default();
        let mut disabled_acc = StyleAccumulator::default();

        for class in classes {
            let (state, actual_class) = parse_state_prefix(class);
            let acc = match state {
                ControlState::Normal => &mut normal_acc,
                ControlState::Hover => &mut hover_acc,
                ControlState::Focus => &mut focus_acc,
                ControlState::Active => &mut active_acc,
                ControlState::Disabled => &mut disabled_acc,
            };
            acc.parse(actual_class, rem_px);
        }

        let normal = normal_acc.build();

        // 构建其他状态：先克隆 normal，再用自己的值覆盖
        let mut hover = normal.clone();
        hover.merge(&hover_acc.build());

        let mut focus = normal.clone();
        focus.merge(&focus_acc.build());

        let mut active = normal.clone();
        active.merge(&active_acc.build());

        let mut disabled = normal.clone();
        disabled.merge(&disabled_acc.build());

        Self {
            normal,
            hover,
            focus,
            active,
            disabled,
        }
    }

    /// 获取指定状态的样式
    #[inline]
    pub fn get(&self, state: ControlState) -> &DecorationStyle {
        match state {
            ControlState::Normal => &self.normal,
            ControlState::Hover => &self.hover,
            ControlState::Focus => &self.focus,
            ControlState::Active => &self.active,
            ControlState::Disabled => &self.disabled,
        }
    }

    /// 解析单个类名 (不支持状态前缀)，追加到 normal 样式
    #[inline]
    pub fn parse_single(&mut self, class: &str) {
        self.parse_single_with_rem(class, 16.0);
    }

    /// 解析单个类名，指定 rem 基准值
    pub fn parse_single_with_rem(&mut self, class: &str, rem_px: f32) {
        let mut acc = StyleAccumulator::default();
        acc.parse(class, rem_px);
        self.normal.merge(&acc.build());
    }
}


// ============================================================================
// Style Accumulator (internal)
// ============================================================================

#[derive(Default)]
struct StyleAccumulator {
    // Border Width
    border_l: Option<f32>,
    border_r: Option<f32>,
    border_t: Option<f32>,
    border_b: Option<f32>,
    border_color: Option<Color>,

    // Border Radius
    radius_tl: Option<f32>,
    radius_tr: Option<f32>,
    radius_br: Option<f32>,
    radius_bl: Option<f32>,

    // Background
    background_color: Option<Color>,

    // Opacity
    opacity: Option<f32>,
}

impl StyleAccumulator {
    fn build(self) -> DecorationStyle {
        DecorationStyle {
            border_width: BorderRect {
                left: self.border_l.unwrap_or(0.0),
                right: self.border_r.unwrap_or(0.0),
                top: self.border_t.unwrap_or(0.0),
                bottom: self.border_b.unwrap_or(0.0),
            },
            border_color: self.border_color,
            border_radius: CornerRadius {
                top_left: self.radius_tl.unwrap_or(0.0),
                top_right: self.radius_tr.unwrap_or(0.0),
                bottom_right: self.radius_br.unwrap_or(0.0),
                bottom_left: self.radius_bl.unwrap_or(0.0),
            },
            background_color: self.background_color,
            opacity: self.opacity,
        }
    }

    fn parse(&mut self, class: &str, rem_px: f32) {
        let class = class.trim();
        if class.is_empty() {
            return;
        }

        // ================================================================
        // Opacity: opacity-*
        // ================================================================
        if let Some(suffix) = class.strip_prefix("opacity-") {
            if let Some(inner) = extract_bracket_value(suffix) {
                if let Ok(v) = inner.parse::<f32>() {
                    self.opacity = Some(v);
                    return;
                }
            }
            if let Ok(v) = suffix.parse::<u8>() {
                self.opacity = Some(v as f32 / 100.0);

                return;
            }
        }

        // ================================================================
        // Background Color: bg-*
        // ================================================================
        if let Some(suffix) = class.strip_prefix("bg-") {
            if let Some(color) = parse_color(suffix) {
                self.background_color = Some(color);

                return;
            }
        }

        // ================================================================
        // Border Color: border-{color}-{shade}
        // ================================================================
        if class.starts_with("border-")
            && !class.starts_with("border-t-")
            && !class.starts_with("border-b-")
            && !class.starts_with("border-l-")
            && !class.starts_with("border-r-")
            && !class.starts_with("border-x-")
            && !class.starts_with("border-y-")
        {
            let suffix = &class[7..];
            if suffix.contains('-')
                || suffix == "transparent"
                || suffix == "black"
                || suffix == "white"
                || suffix.starts_with('[')
            {
                if let Some(color) = parse_color(suffix) {
                    self.border_color = Some(color);

                    return;
                }
            }
        }

        // ================================================================
        // Border Width
        // ================================================================
        if class == "border" {
            self.border_l = Some(1.0);
            self.border_r = Some(1.0);
            self.border_t = Some(1.0);
            self.border_b = Some(1.0);
            return;
        }

        // border-t/b/l/r-*
        if let Some(suffix) = class.strip_prefix("border-t-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length(v, rem_px))
                .or_else(|| suffix.parse::<f32>().ok())
            {
                self.border_t = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("border-b-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length(v, rem_px))
                .or_else(|| suffix.parse::<f32>().ok())
            {
                self.border_b = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("border-l-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length(v, rem_px))
                .or_else(|| suffix.parse::<f32>().ok())
            {
                self.border_l = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("border-r-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length(v, rem_px))
                .or_else(|| suffix.parse::<f32>().ok())
            {
                self.border_r = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("border-x-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length(v, rem_px))
                .or_else(|| suffix.parse::<f32>().ok())
            {
                self.border_l = Some(v);
                self.border_r = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("border-y-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length(v, rem_px))
                .or_else(|| suffix.parse::<f32>().ok())
            {
                self.border_t = Some(v);
                self.border_b = Some(v);

                return;
            }
        }
        // border-* (通用宽度)
        if let Some(suffix) = class.strip_prefix("border-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length(v, rem_px))
                .or_else(|| suffix.parse::<f32>().ok())
            {
                self.border_l = Some(v);
                self.border_r = Some(v);
                self.border_t = Some(v);
                self.border_b = Some(v);

                return;
            }
        }

        // ================================================================
        // Border Radius
        // ================================================================
        if class == "rounded" {
            let v = 4.0;
            self.radius_tl = Some(v);
            self.radius_tr = Some(v);
            self.radius_br = Some(v);
            self.radius_bl = Some(v);
            return;
        }
        if class == "rounded-none" {
            self.radius_tl = Some(0.0);
            self.radius_tr = Some(0.0);
            self.radius_br = Some(0.0);
            self.radius_bl = Some(0.0);
            return;
        }
        if class == "rounded-full" {
            self.radius_tl = Some(9999.0);
            self.radius_tr = Some(9999.0);
            self.radius_br = Some(9999.0);
            self.radius_bl = Some(9999.0);
            return;
        }

        // 预设圆角
        let radius_presets: &[(&str, f32)] = &[
            ("rounded-sm", 2.0),
            ("rounded-md", 6.0),
            ("rounded-lg", 8.0),
            ("rounded-xl", 12.0),
            ("rounded-2xl", 16.0),
            ("rounded-3xl", 24.0),
        ];
        for (preset, value) in radius_presets {
            if class == *preset {
                self.radius_tl = Some(*value);
                self.radius_tr = Some(*value);
                self.radius_br = Some(*value);
                self.radius_bl = Some(*value);

                return;
            }
        }

        // rounded-t/b/l/r-* (双角)
        if let Some(suffix) = class.strip_prefix("rounded-t-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_tl = Some(v);
                self.radius_tr = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("rounded-b-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_bl = Some(v);
                self.radius_br = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("rounded-l-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_tl = Some(v);
                self.radius_bl = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("rounded-r-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_tr = Some(v);
                self.radius_br = Some(v);

                return;
            }
        }

        // rounded-tl/tr/bl/br-* (单角)
        if let Some(suffix) = class.strip_prefix("rounded-tl-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_tl = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("rounded-tr-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_tr = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("rounded-bl-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_bl = Some(v);

                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("rounded-br-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_br = Some(v);

                return;
            }
        }

        // rounded-[Npx] (任意值)
        if let Some(suffix) = class.strip_prefix("rounded-") {
            if let Some(v) = Self::parse_radius_value(suffix, rem_px) {
                self.radius_tl = Some(v);
                self.radius_tr = Some(v);
                self.radius_br = Some(v);
                self.radius_bl = Some(v);
            }
        }
    }

    fn parse_radius_value(value: &str, rem_px: f32) -> Option<f32> {
        match value {
            "none" => Some(0.0),
            "sm" => Some(2.0),
            "md" => Some(6.0),
            "lg" => Some(8.0),
            "xl" => Some(12.0),
            "2xl" => Some(16.0),
            "3xl" => Some(24.0),
            "full" => Some(9999.0),
            _ => {
                if let Some(inner) = extract_bracket_value(value) {
                    parse_length(inner, rem_px)
                } else {
                    value.parse::<f32>().ok()
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to compare colors by RGB values
    fn assert_color_eq(actual: Option<Color>, expected_r: u8, expected_g: u8, expected_b: u8) {
        let color = actual.expect("Expected color to be Some");
        assert_eq!(color.r, expected_r, "Red channel mismatch");
        assert_eq!(color.g, expected_g, "Green channel mismatch");
        assert_eq!(color.b, expected_b, "Blue channel mismatch");
    }

    #[test]
    fn test_simple_parse() {
        let deco = Decoration::parse(&["bg-white", "border", "border-gray-300", "rounded-lg"]);

        assert!(deco.normal.background_color.is_some());
        assert_color_eq(deco.normal.background_color, 255, 255, 255); // WHITE
        assert_eq!(deco.normal.border_width.left, 1.0);
        assert!(deco.normal.border_color.is_some());
        assert_eq!(deco.normal.border_radius.top_left, 8.0);
    }

    #[test]
    fn test_hover_state() {
        let deco = Decoration::parse(&["bg-white", "hover:bg-gray-100"]);

        assert_color_eq(deco.normal.background_color, 255, 255, 255); // WHITE
        // hover 继承 normal 并覆盖
        assert_color_eq(deco.hover.background_color, 243, 244, 246); // GRAY_100
    }

    #[test]
    fn test_state_inheritance() {
        let deco = Decoration::parse(&["bg-blue-500", "rounded-md"]);

        // 没有设置 hover 前缀，但 hover 继承了 normal
        assert_color_eq(deco.normal.background_color, 59, 130, 246); // BLUE_500
        assert_color_eq(deco.hover.background_color, 59, 130, 246); // 继承 normal
        assert_color_eq(deco.focus.background_color, 59, 130, 246); // 继承 normal
        assert_eq!(deco.normal.border_radius.top_left, 6.0);
        assert_eq!(deco.hover.border_radius.top_left, 6.0); // 继承
    }

    #[test]
    fn test_hex_color() {
        let deco = Decoration::parse(&["bg-[#ff6600]"]);

        let color = deco.normal.background_color.unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 102);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_opacity() {
        let deco = Decoration::parse(&["opacity-50"]);
        assert!((deco.normal.get_opacity() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_get_state() {
        let deco = Decoration::parse(&["bg-white", "hover:bg-gray-100"]);

        // get() 方法应该正确返回
        let normal = deco.get(ControlState::Normal);
        let hover = deco.get(ControlState::Hover);
        let focus = deco.get(ControlState::Focus);

        assert_color_eq(normal.background_color, 255, 255, 255);
        assert_color_eq(hover.background_color, 243, 244, 246);
        assert_color_eq(focus.background_color, 255, 255, 255); // 继承 normal
    }

    #[test]
    fn test_parse_single() {
        let mut deco = Decoration::new();
        deco.parse_single("bg-blue-500");
        deco.parse_single("rounded-lg");

        assert!(deco.normal.background_color.is_some());
        assert_eq!(deco.normal.border_radius.top_left, 8.0);
    }
}

