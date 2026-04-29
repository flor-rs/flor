use crate::types::Rect;

#[cfg(feature = "text-layout")]
mod handle;
#[cfg(feature = "text-layout")]
pub use handle::*;

#[cfg(feature = "text-layout")]
mod layout;
#[cfg(feature = "text-layout")]
pub use layout::*;

/// 对应 CSS font-weight
/// 数值参考: https://developer.mozilla.org/en-US/docs/Web/CSS/font-weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontWeight {
    Thin,       // 100
    ExtraLight, // 200
    Light,      // 300
    #[default]
    Normal, // 400 (Regular)
    Medium,     // 500
    SemiBold,   // 600
    Bold,       // 700
    ExtraBold,  // 800
    Black,      // 900
    ExtraBlack, // 950
}

/// 对应 CSS font-style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Oblique,
    Italic,
}

/// 对应 CSS font-stretch
/// 用于支持压缩或扩展字体的字形
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    #[default]
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

/// 对应 CSS text-align (水平方向)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignment {
    #[default]
    Start, // Left (in LTR)
    End, // Right (in LTR)
    Center,
    Justified, // 两端对齐
}

/// 对应 CSS vertical-align 或 Flexbox 的 align-items (垂直方向)
/// 在 draw_text 的矩形框内，文字是靠上、居中还是靠下
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParagraphAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

/// 对应 CSS white-space / word-break
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordWrapping {
    NoWrap, // 不换行
    #[default]
    Wrap, // 按单词换行 (Word boundary)
    Character, // 强制按字符换行 (主要用于 CJK 或长单词打断)
}

/// 对应 CSS text-overflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextTrimming {
    #[default]
    None, // 不做处理，直接绘制（可能会超出边界或被 Clip）
    Character,    // 在字符处截断
    Word,         // 在单词处截断
    EllipsisChar, // 在字符处截断并显示 "..." (text-overflow: ellipsis)
    EllipsisWord, // 在单词处截断并显示 "..."
}

/// 命中测试结果
pub struct HitTestResult {
    /// 字符在全文中的字节偏移量 (UTF-8 Byte Index)
    pub global_index: usize,
    /// 行号 (从 0 开始)
    pub line: usize,
    /// 字符在当前行内的字节偏移量 (UTF-8 Byte Index)
    pub line_offset: usize,
    /// 命中位置是否位于字符的后半部分
    /// 对于 D2D：对应 isTrailingHit
    /// 对于 cosmic-text：对应 Affinity::After
    pub is_trailing: bool,
    /// 命中的字符在 UI 坐标系中的包围盒
    pub bounds: Option<Rect<f32>>,
    /// 鼠标点击是否落在文本的物理区域内
    pub is_inside: bool,
}
