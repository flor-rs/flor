use crate::graphics::color::Color;

/// 阴影参数结构，可用于矩形和文本阴影
#[derive(Copy, Clone, Debug, Default)]
pub struct Shadow {
    /// 阴影在 X 方向的偏移
    pub offset_x: f32,
    /// 阴影在 Y 方向的偏移
    pub offset_y: f32,
    /// 阴影模糊半径
    pub blur_radius: f32,
    /// 控制阴影扩展
    pub spread: f32,
    /// 阴影颜色
    pub color: Color,
    /// true = 内阴影, false = 外阴影
    pub inset: bool,
}
