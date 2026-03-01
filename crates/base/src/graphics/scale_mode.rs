/// 定义了图像在目标矩形内渲染时的缩放方式。
///
/// 这些模式决定了图像的原始纵横比是否应被保持，以及图像与目标区域
/// 之间的适应或填充关系。
///
/// 行为模式概览：
/// - **Fit**：等比缩放，保证图像完整可见。
/// - **Cover**：等比缩放，保证目标区域被图像完全覆盖（可能裁剪图像）。
/// - **Stretch**：非等比拉伸，保证图像完全填满目标区域（可能导致图像失真）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScaleMode {
    /// 保持图像的原始尺寸，不进行任何缩放。
    /// 图像将居于目标矩形的左上角。
    None,

    /// **等比适应 (Aspect Fit / Contain)**。
    /// 保持图像的纵横比，将其缩放至刚好适应目标矩形内部的最大尺寸。
    /// 图像将**完整可见**，目标矩形内可能留有空白。
    Fit,

    /// **等比覆盖 (Aspect Fill / Cover)**。
    /// 保持图像的纵横比，将其缩放至刚好能完全**覆盖**目标矩形所需的最小尺寸。
    /// 目标矩形将被完全填满，但图像超出目标区域的部分会被**裁剪**。
    Cover,

    /// **非等比拉伸填满 (Non-uniform Stretch Fill)**。
    /// 忽略图像的纵横比，强制拉伸图像以精确匹配目标矩形的长宽。
    /// 图像将完全填满目标区域，但可能会导致图像**失真**。
    Stretch,

    /// 保持图像的原始尺寸，不进行缩放，并将图像**居中**显示在目标矩形内。
    /// 如果图像大于目标矩形，它将超出边界。
    Center,
}

impl ScaleMode {
    /// 计算缩放后的绘制区域
    /// x, y: 容器起始坐标
    /// target_w, target_h: 容器尺寸
    /// content_w, content_h: 内容原始尺寸 (对于你的 SVG 来说就是 intrinsic * dpi)
    pub fn calc_draw_rect(
        &self,
        x: f32,
        y: f32,
        target_w: f32,
        target_h: f32,
        content_w: f32,
        content_h: f32,
    ) -> (f32, f32, f32, f32) {
        let mut draw_x = x;
        let mut draw_y = y;
        let mut draw_w = target_w;
        let mut draw_h = target_h;

        match self {
            ScaleMode::None => {
                draw_w = content_w;
                draw_h = content_h;
            }
            ScaleMode::Fit | ScaleMode::Cover => {
                let scale_x = target_w / content_w;
                let scale_y = target_h / content_h;
                let scale = if matches!(self, ScaleMode::Fit) {
                    scale_x.min(scale_y)
                } else {
                    scale_x.max(scale_y)
                };
                draw_w = content_w * scale;
                draw_h = content_h * scale;
                draw_x = x + (target_w - draw_w) / 2.0;
                draw_y = y + (target_h - draw_h) / 2.0;
            }
            ScaleMode::Center => {
                draw_w = content_w;
                draw_h = content_h;
                draw_x = x + (target_w - draw_w) / 2.0;
                draw_y = y + (target_h - draw_h) / 2.0;
            }
            ScaleMode::Stretch => {}
        }

        (draw_x, y, draw_w, draw_h)
    }
}
