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

use flor_graphics_base::Color;

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
                DecorationState::Normal => &mut normal_acc,
                DecorationState::Hover => &mut hover_acc,
                DecorationState::Focus => &mut focus_acc,
                DecorationState::Active => &mut active_acc,
                DecorationState::Disabled => &mut disabled_acc,
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
    pub fn get(&self, state: DecorationState) -> &DecorationStyle {
        match state {
            DecorationState::Normal => &self.normal,
            DecorationState::Hover => &self.hover,
            DecorationState::Focus => &self.focus,
            DecorationState::Active => &self.active,
            DecorationState::Disabled => &self.disabled,
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

/// 装饰状态枚举
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DecorationState {
    #[default]
    Normal,
    Hover,
    Focus,
    Active,
    Disabled,
}

// ============================================================================
// Internal Parser
// ============================================================================

/// Parse state prefix from class name
#[inline]
fn parse_state_prefix(class: &str) -> (DecorationState, &str) {
    if let Some(rest) = class.strip_prefix("hover:") {
        (DecorationState::Hover, rest)
    } else if let Some(rest) = class.strip_prefix("focus:") {
        (DecorationState::Focus, rest)
    } else if let Some(rest) = class.strip_prefix("active:") {
        (DecorationState::Active, rest)
    } else if let Some(rest) = class.strip_prefix("disabled:") {
        (DecorationState::Disabled, rest)
    } else {
        (DecorationState::Normal, class)
    }
}

#[inline]
fn extract_bracket_value(s: &str) -> Option<&str> {
    if s.starts_with('[') && s.ends_with(']') {
        Some(&s[1..s.len() - 1])
    } else {
        None
    }
}

/// 解析长度值 (px, rem, 或纯数字)
#[inline]
fn parse_length(value: &str, rem_px: f32) -> Option<f32> {
    if let Some(v) = value.strip_suffix("px") {
        return v.parse::<f32>().ok();
    }
    if let Some(v) = value.strip_suffix("rem") {
        return v.parse::<f32>().ok().map(|n| n * rem_px);
    }
    value.parse::<f32>().ok()
}

/// 解析颜色值
fn parse_color(value: &str) -> Option<Color> {
    // 关键字
    match value {
        "transparent" => return Some(Color::rgba(0, 0, 0, 0)),
        "black" => return Some(Color::BLACK),
        "white" => return Some(Color::WHITE),
        _ => {}
    }

    // #hex 或 [#hex]
    if value.starts_with('#') {
        return Color::from_hex_str(value).ok();
    }
    if let Some(inner) = extract_bracket_value(value) {
        if inner.starts_with('#') {
            return Color::from_hex_str(inner).ok();
        }
    }

    // TW 颜色名 (color-shade)
    if let Some(idx) = value.rfind('-') {
        let color_name = &value[..idx];
        let shade = &value[idx + 1..];
        return parse_tw_color(color_name, shade);
    }

    None
}

/// 解析 TW 颜色名
fn parse_tw_color(color_name: &str, shade: &str) -> Option<Color> {
    let shade_num: u16 = shade.parse().ok()?;

    match (color_name, shade_num) {
        // Slate
        ("slate", 50) => Some(Color::SLATE_50),
        ("slate", 100) => Some(Color::SLATE_100),
        ("slate", 200) => Some(Color::SLATE_200),
        ("slate", 300) => Some(Color::SLATE_300),
        ("slate", 400) => Some(Color::SLATE_400),
        ("slate", 500) => Some(Color::SLATE_500),
        ("slate", 600) => Some(Color::SLATE_600),
        ("slate", 700) => Some(Color::SLATE_700),
        ("slate", 800) => Some(Color::SLATE_800),
        ("slate", 900) => Some(Color::SLATE_900),
        ("slate", 950) => Some(Color::SLATE_950),
        // Gray
        ("gray", 50) => Some(Color::GRAY_50),
        ("gray", 100) => Some(Color::GRAY_100),
        ("gray", 200) => Some(Color::GRAY_200),
        ("gray", 300) => Some(Color::GRAY_300),
        ("gray", 400) => Some(Color::GRAY_400),
        ("gray", 500) => Some(Color::GRAY_500),
        ("gray", 600) => Some(Color::GRAY_600),
        ("gray", 700) => Some(Color::GRAY_700),
        ("gray", 800) => Some(Color::GRAY_800),
        ("gray", 900) => Some(Color::GRAY_900),
        ("gray", 950) => Some(Color::GRAY_950),
        // Zinc
        ("zinc", 50) => Some(Color::ZINC_50),
        ("zinc", 100) => Some(Color::ZINC_100),
        ("zinc", 200) => Some(Color::ZINC_200),
        ("zinc", 300) => Some(Color::ZINC_300),
        ("zinc", 400) => Some(Color::ZINC_400),
        ("zinc", 500) => Some(Color::ZINC_500),
        ("zinc", 600) => Some(Color::ZINC_600),
        ("zinc", 700) => Some(Color::ZINC_700),
        ("zinc", 800) => Some(Color::ZINC_800),
        ("zinc", 900) => Some(Color::ZINC_900),
        ("zinc", 950) => Some(Color::ZINC_950),
        // Neutral
        ("neutral", 50) => Some(Color::NEUTRAL_50),
        ("neutral", 100) => Some(Color::NEUTRAL_100),
        ("neutral", 200) => Some(Color::NEUTRAL_200),
        ("neutral", 300) => Some(Color::NEUTRAL_300),
        ("neutral", 400) => Some(Color::NEUTRAL_400),
        ("neutral", 500) => Some(Color::NEUTRAL_500),
        ("neutral", 600) => Some(Color::NEUTRAL_600),
        ("neutral", 700) => Some(Color::NEUTRAL_700),
        ("neutral", 800) => Some(Color::NEUTRAL_800),
        ("neutral", 900) => Some(Color::NEUTRAL_900),
        ("neutral", 950) => Some(Color::NEUTRAL_950),
        // Red
        ("red", 50) => Some(Color::RED_50),
        ("red", 100) => Some(Color::RED_100),
        ("red", 200) => Some(Color::RED_200),
        ("red", 300) => Some(Color::RED_300),
        ("red", 400) => Some(Color::RED_400),
        ("red", 500) => Some(Color::RED_500),
        ("red", 600) => Some(Color::RED_600),
        ("red", 700) => Some(Color::RED_700),
        ("red", 800) => Some(Color::RED_800),
        ("red", 900) => Some(Color::RED_900),
        ("red", 950) => Some(Color::RED_950),
        // Orange
        ("orange", 50) => Some(Color::ORANGE_50),
        ("orange", 100) => Some(Color::ORANGE_100),
        ("orange", 200) => Some(Color::ORANGE_200),
        ("orange", 300) => Some(Color::ORANGE_300),
        ("orange", 400) => Some(Color::ORANGE_400),
        ("orange", 500) => Some(Color::ORANGE_500),
        ("orange", 600) => Some(Color::ORANGE_600),
        ("orange", 700) => Some(Color::ORANGE_700),
        ("orange", 800) => Some(Color::ORANGE_800),
        ("orange", 900) => Some(Color::ORANGE_900),
        ("orange", 950) => Some(Color::ORANGE_950),
        // Amber
        ("amber", 50) => Some(Color::AMBER_50),
        ("amber", 100) => Some(Color::AMBER_100),
        ("amber", 200) => Some(Color::AMBER_200),
        ("amber", 300) => Some(Color::AMBER_300),
        ("amber", 400) => Some(Color::AMBER_400),
        ("amber", 500) => Some(Color::AMBER_500),
        ("amber", 600) => Some(Color::AMBER_600),
        ("amber", 700) => Some(Color::AMBER_700),
        ("amber", 800) => Some(Color::AMBER_800),
        ("amber", 900) => Some(Color::AMBER_900),
        ("amber", 950) => Some(Color::AMBER_950),
        // Yellow
        ("yellow", 50) => Some(Color::YELLOW_50),
        ("yellow", 100) => Some(Color::YELLOW_100),
        ("yellow", 200) => Some(Color::YELLOW_200),
        ("yellow", 300) => Some(Color::YELLOW_300),
        ("yellow", 400) => Some(Color::YELLOW_400),
        ("yellow", 500) => Some(Color::YELLOW_500),
        ("yellow", 600) => Some(Color::YELLOW_600),
        ("yellow", 700) => Some(Color::YELLOW_700),
        ("yellow", 800) => Some(Color::YELLOW_800),
        ("yellow", 900) => Some(Color::YELLOW_900),
        ("yellow", 950) => Some(Color::YELLOW_950),
        // Lime
        ("lime", 50) => Some(Color::LIME_50),
        ("lime", 100) => Some(Color::LIME_100),
        ("lime", 200) => Some(Color::LIME_200),
        ("lime", 300) => Some(Color::LIME_300),
        ("lime", 400) => Some(Color::LIME_400),
        ("lime", 500) => Some(Color::LIME_500),
        ("lime", 600) => Some(Color::LIME_600),
        ("lime", 700) => Some(Color::LIME_700),
        ("lime", 800) => Some(Color::LIME_800),
        ("lime", 900) => Some(Color::LIME_900),
        ("lime", 950) => Some(Color::LIME_950),
        // Green
        ("green", 50) => Some(Color::GREEN_50),
        ("green", 100) => Some(Color::GREEN_100),
        ("green", 200) => Some(Color::GREEN_200),
        ("green", 300) => Some(Color::GREEN_300),
        ("green", 400) => Some(Color::GREEN_400),
        ("green", 500) => Some(Color::GREEN_500),
        ("green", 600) => Some(Color::GREEN_600),
        ("green", 700) => Some(Color::GREEN_700),
        ("green", 800) => Some(Color::GREEN_800),
        ("green", 900) => Some(Color::GREEN_900),
        ("green", 950) => Some(Color::GREEN_950),
        // Emerald
        ("emerald", 50) => Some(Color::EMERALD_50),
        ("emerald", 100) => Some(Color::EMERALD_100),
        ("emerald", 200) => Some(Color::EMERALD_200),
        ("emerald", 300) => Some(Color::EMERALD_300),
        ("emerald", 400) => Some(Color::EMERALD_400),
        ("emerald", 500) => Some(Color::EMERALD_500),
        ("emerald", 600) => Some(Color::EMERALD_600),
        ("emerald", 700) => Some(Color::EMERALD_700),
        ("emerald", 800) => Some(Color::EMERALD_800),
        ("emerald", 900) => Some(Color::EMERALD_900),
        ("emerald", 950) => Some(Color::EMERALD_950),
        // Teal
        ("teal", 50) => Some(Color::TEAL_50),
        ("teal", 100) => Some(Color::TEAL_100),
        ("teal", 200) => Some(Color::TEAL_200),
        ("teal", 300) => Some(Color::TEAL_300),
        ("teal", 400) => Some(Color::TEAL_400),
        ("teal", 500) => Some(Color::TEAL_500),
        ("teal", 600) => Some(Color::TEAL_600),
        ("teal", 700) => Some(Color::TEAL_700),
        ("teal", 800) => Some(Color::TEAL_800),
        ("teal", 900) => Some(Color::TEAL_900),
        ("teal", 950) => Some(Color::TEAL_950),
        // Cyan
        ("cyan", 50) => Some(Color::CYAN_50),
        ("cyan", 100) => Some(Color::CYAN_100),
        ("cyan", 200) => Some(Color::CYAN_200),
        ("cyan", 300) => Some(Color::CYAN_300),
        ("cyan", 400) => Some(Color::CYAN_400),
        ("cyan", 500) => Some(Color::CYAN_500),
        ("cyan", 600) => Some(Color::CYAN_600),
        ("cyan", 700) => Some(Color::CYAN_700),
        ("cyan", 800) => Some(Color::CYAN_800),
        ("cyan", 900) => Some(Color::CYAN_900),
        ("cyan", 950) => Some(Color::CYAN_950),
        // Sky
        ("sky", 50) => Some(Color::SKY_50),
        ("sky", 100) => Some(Color::SKY_100),
        ("sky", 200) => Some(Color::SKY_200),
        ("sky", 300) => Some(Color::SKY_300),
        ("sky", 400) => Some(Color::SKY_400),
        ("sky", 500) => Some(Color::SKY_500),
        ("sky", 600) => Some(Color::SKY_600),
        ("sky", 700) => Some(Color::SKY_700),
        ("sky", 800) => Some(Color::SKY_800),
        ("sky", 900) => Some(Color::SKY_900),
        ("sky", 950) => Some(Color::SKY_950),
        // Blue
        ("blue", 50) => Some(Color::BLUE_50),
        ("blue", 100) => Some(Color::BLUE_100),
        ("blue", 200) => Some(Color::BLUE_200),
        ("blue", 300) => Some(Color::BLUE_300),
        ("blue", 400) => Some(Color::BLUE_400),
        ("blue", 500) => Some(Color::BLUE_500),
        ("blue", 600) => Some(Color::BLUE_600),
        ("blue", 700) => Some(Color::BLUE_700),
        ("blue", 800) => Some(Color::BLUE_800),
        ("blue", 900) => Some(Color::BLUE_900),
        ("blue", 950) => Some(Color::BLUE_950),
        // Indigo
        ("indigo", 50) => Some(Color::INDIGO_50),
        ("indigo", 100) => Some(Color::INDIGO_100),
        ("indigo", 200) => Some(Color::INDIGO_200),
        ("indigo", 300) => Some(Color::INDIGO_300),
        ("indigo", 400) => Some(Color::INDIGO_400),
        ("indigo", 500) => Some(Color::INDIGO_500),
        ("indigo", 600) => Some(Color::INDIGO_600),
        ("indigo", 700) => Some(Color::INDIGO_700),
        ("indigo", 800) => Some(Color::INDIGO_800),
        ("indigo", 900) => Some(Color::INDIGO_900),
        ("indigo", 950) => Some(Color::INDIGO_950),
        // Violet
        ("violet", 50) => Some(Color::VIOLET_50),
        ("violet", 100) => Some(Color::VIOLET_100),
        ("violet", 200) => Some(Color::VIOLET_200),
        ("violet", 300) => Some(Color::VIOLET_300),
        ("violet", 400) => Some(Color::VIOLET_400),
        ("violet", 500) => Some(Color::VIOLET_500),
        ("violet", 600) => Some(Color::VIOLET_600),
        ("violet", 700) => Some(Color::VIOLET_700),
        ("violet", 800) => Some(Color::VIOLET_800),
        ("violet", 900) => Some(Color::VIOLET_900),
        ("violet", 950) => Some(Color::VIOLET_950),
        // Purple
        ("purple", 50) => Some(Color::PURPLE_50),
        ("purple", 100) => Some(Color::PURPLE_100),
        ("purple", 200) => Some(Color::PURPLE_200),
        ("purple", 300) => Some(Color::PURPLE_300),
        ("purple", 400) => Some(Color::PURPLE_400),
        ("purple", 500) => Some(Color::PURPLE_500),
        ("purple", 600) => Some(Color::PURPLE_600),
        ("purple", 700) => Some(Color::PURPLE_700),
        ("purple", 800) => Some(Color::PURPLE_800),
        ("purple", 900) => Some(Color::PURPLE_900),
        ("purple", 950) => Some(Color::PURPLE_950),
        // Fuchsia
        ("fuchsia", 50) => Some(Color::FUCHSIA_50),
        ("fuchsia", 100) => Some(Color::FUCHSIA_100),
        ("fuchsia", 200) => Some(Color::FUCHSIA_200),
        ("fuchsia", 300) => Some(Color::FUCHSIA_300),
        ("fuchsia", 400) => Some(Color::FUCHSIA_400),
        ("fuchsia", 500) => Some(Color::FUCHSIA_500),
        ("fuchsia", 600) => Some(Color::FUCHSIA_600),
        ("fuchsia", 700) => Some(Color::FUCHSIA_700),
        ("fuchsia", 800) => Some(Color::FUCHSIA_800),
        ("fuchsia", 900) => Some(Color::FUCHSIA_900),
        ("fuchsia", 950) => Some(Color::FUCHSIA_950),
        // Pink
        ("pink", 50) => Some(Color::PINK_50),
        ("pink", 100) => Some(Color::PINK_100),
        ("pink", 200) => Some(Color::PINK_200),
        ("pink", 300) => Some(Color::PINK_300),
        ("pink", 400) => Some(Color::PINK_400),
        ("pink", 500) => Some(Color::PINK_500),
        ("pink", 600) => Some(Color::PINK_600),
        ("pink", 700) => Some(Color::PINK_700),
        ("pink", 800) => Some(Color::PINK_800),
        ("pink", 900) => Some(Color::PINK_900),
        ("pink", 950) => Some(Color::PINK_950),
        // Rose
        ("rose", 50) => Some(Color::ROSE_50),
        ("rose", 100) => Some(Color::ROSE_100),
        ("rose", 200) => Some(Color::ROSE_200),
        ("rose", 300) => Some(Color::ROSE_300),
        ("rose", 400) => Some(Color::ROSE_400),
        ("rose", 500) => Some(Color::ROSE_500),
        ("rose", 600) => Some(Color::ROSE_600),
        ("rose", 700) => Some(Color::ROSE_700),
        ("rose", 800) => Some(Color::ROSE_800),
        ("rose", 900) => Some(Color::ROSE_900),
        ("rose", 950) => Some(Color::ROSE_950),
        _ => None,
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
        let normal = deco.get(DecorationState::Normal);
        let hover = deco.get(DecorationState::Hover);
        let focus = deco.get(DecorationState::Focus);

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

