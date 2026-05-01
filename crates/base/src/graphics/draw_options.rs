use crate::graphics::{ScaleMode, Shadow};
use crate::types::Transform2D;

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

pub type SurfaceDrawOptions = ImageDrawOptions;

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
    pub opacity: Option<f32>,
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

/// 文本高亮块，用于编辑器语法高亮等场景
#[derive(Clone, Debug)]
pub struct TextChunk<B, TF> {
    /// 该文本块使用的画笔
    pub brush: B,
    pub text_format: TF,
    /// 起始字节索引（和标准库 find 方法返回的索引单位一致）
    pub start: usize,
    /// 字节长度
    pub length: usize,
}
