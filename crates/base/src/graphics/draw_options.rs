use crate::graphics::{ScaleMode, Shadow};
use crate::types::transform2d::Transform2D;

/// 图片绘制可选参数
#[derive(Clone, Debug, Default)]
pub struct ImageDrawOptions {
    /// 可选缩放模式（Fit / Fill / Stretch 等）
    pub scale_mode: Option<ScaleMode>,
    /// 可选矩阵变换
    pub transform: Option<Transform2D>,
    /// 可选阴影
    pub shadow: Option<Shadow>,
    pub frame_index: Option<usize>,
    pub opacity: Option<f32>,
}

/// SVG 绘制可选参数
#[cfg(feature = "svg")]
#[derive(Clone, Debug, Default)]
pub struct SvgDrawOptions {
    /// 可选缩放模式
    pub scale_mode: Option<ScaleMode>,
    /// 可选矩阵变换
    pub transform: Option<Transform2D>,
    /// 可选阴影
    pub shadow: Option<Shadow>,
}

/// Path 绘制可选参数
#[derive(Clone, Debug, Default)]
pub struct PathDrawOptions {
    /// 可选矩阵变换
    pub transform: Option<Transform2D>,
    /// 可选阴影
    pub shadow: Option<Shadow>,
}

/// 文本绘制可选参数
#[derive(Clone, Debug, Default)]
pub struct TextDrawOptions {
    /// 可选矩阵变换
    pub transform: Option<Transform2D>,
    /// 可选阴影
    pub shadow: Option<Shadow>,
}
