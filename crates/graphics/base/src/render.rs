#[cfg(feature = "svg")]
use crate::SvgDrawOptions;
#[cfg(feature = "svg")]
use crate::SvgHandle;
use crate::{
    BrushHandle, Color, Gradient, HitTestResult, ImageDrawOptions, ImageHandle, Path,
    PathDrawOptions, SurfaceId, TextDrawOptions, TextFormatHandle, Transform2D,
};
use std::any::Any;

pub trait Render: RenderContext {
    type HWND;
    type Render;

    /// 创建渲染上下文
    fn create(
        hwnd: impl Into<Self::HWND>,
        width: u32,
        height: u32,
        wait_v_sync: bool,
    ) -> Result<Self::Render, Self::Error>;
}

/// 基础渲染 trait，定义统一绘制能力
/// 适用于 CPU / GPU 后端，支持跨平台绘制
pub trait RenderContext: Any {
    type Error: std::error::Error;
    type ImageHandle: ImageHandle;
    type SurfaceId: SurfaceId;
    type BrushHandle: BrushHandle;
    #[cfg(feature = "svg")]
    type SvgHandle: SvgHandle;
    type TextFormatHandle: TextFormatHandle;

    // ==================== 帧管理 ====================
    /// 开始渲染帧
    fn begin(&mut self) -> Result<(), Self::Error>;
    /// 结束渲染帧
    fn end(&mut self) -> Result<(), Self::Error>;
    /// 清屏
    fn clear(&mut self, color: Color) -> Result<(), Self::Error>;
    /// 测试方法，可用于调试
    fn test(&mut self) -> Result<(), Self::Error>;
    /// 更新渲染目标尺寸
    fn update_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error>;
    fn set_scale_factor(&mut self, dpi_x: f64, dpi_y: f64) -> Result<(), Self::Error>;

    // ==================== 资源管理 ====================
    fn create_surface(&mut self, width: u32, height: u32) -> Result<Self::SurfaceId, Self::Error>;

    /// 为None时，切换回窗口
    fn set_render_target(&mut self, surface_id: &Self::SurfaceId) -> Result<(), Self::Error>;
    fn reset_render_target(&mut self) -> Result<(), Self::Error>;

    // ==================== 资源管理 ====================
    /// 创建图片资源
    fn create_image_from_bytes(&mut self, bytes: &[u8]) -> Result<Self::ImageHandle, Self::Error>;
    fn create_image_from_raw_bytes(
        &mut self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<Self::ImageHandle, Self::Error>;

    /// 创建 SVG 资源
    #[cfg(feature = "svg")]
    fn create_svg(&mut self, bytes: &[u8]) -> Result<Self::SvgHandle, Self::Error>;

    /// 创建文本格式对象
    fn create_text_format(
        &mut self,
        font_family_name: &str,
    ) -> Result<Self::TextFormatHandle, Self::Error>;

    #[cfg(feature = "memory-font")]
    fn create_text_format_from_bytes(
        &mut self,
        font_data: &[u8],
        ttc_index: u32,
    ) -> Result<Self::TextFormatHandle, Self::Error>;

    /// 测量文本宽高
    fn measure_text(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
    ) -> Result<(f32, f32), Self::Error>;
    /// 像素点 -> 字符索引
    fn hit_test_point(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
        x: f32,
        y: f32,
    ) -> Result<HitTestResult, Self::Error>;

    /// 字符索引 -> 像素点（光标）
    fn hit_test_text_position(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
        text_index: usize,
        trailing: bool,
    ) -> Result<(f32, f32), Self::Error>;

    /// 创建 Brush
    fn create_solid_color_brush(
        &mut self,
        color: Color,
        opacity: Option<f32>,
    ) -> Result<Self::BrushHandle, Self::Error>;
    fn create_gradient_brush(
        &mut self,
        gradient: &Gradient,
    ) -> Result<Self::BrushHandle, Self::Error>;

    // ==================== 绘制方法 ====================
    // ---- 图片 / SVG ----
    fn draw_image(
        &mut self,
        handle: &Self::ImageHandle,
        x: f32,
        y: f32,
        width: Option<f32>,
        height: Option<f32>,
        options: Option<&ImageDrawOptions>,
    ) -> Result<(), Self::Error>;

    #[cfg(feature = "svg")]
    fn draw_svg(
        &mut self,
        handle: &Self::SvgHandle,
        x: f32,
        y: f32,
        width: Option<f32>,
        height: Option<f32>,
        options: Option<&SvgDrawOptions>,
    ) -> Result<(), Self::Error>;

    // ---- 文本 ----
    fn draw_text(
        &mut self,
        text: &str,
        text_format: &mut Self::TextFormatHandle,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        brush: &Self::BrushHandle,
        options: Option<&TextDrawOptions>,
    ) -> Result<(), Self::Error>;

    // ---- 矢量路径 ----
    fn draw_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        stroke_width: f32,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error>;
    fn fill_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error>;

    // ---- 矩形 ----
    fn draw_quad(
        &mut self,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        border_width: f32,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error>;
    fn fill_quad(
        &mut self,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        brush: &Self::BrushHandle,
        corner_radius: Option<f32>,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error>;

    // ---- 特殊效果 ----
    fn blur_quad(
        &mut self,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        corner_radius: f32,
        blur_radius: f32,
        transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error>;

    fn blur_path(
        &mut self,
        path: &Path,
        blur_radius: f32,
        transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error>;

    // ---- 剪裁 ----
    // 1. 基础矩形剪裁 (性能：极快)
    // 对应 tiny-skia 的 clip_rect
    // 对应 GL 的 glScissor
    fn push_clip(&mut self, rect: (f32, f32, f32, f32)) -> Result<(), Self::Error>;

    // 2. 圆角矩形剪裁 (性能：中等)
    // 对应 tiny-skia 的 clip_path (但在 skia 中有专门的 RRect 优化)
    fn push_rounded_clip(
        &mut self,
        rect: (f32, f32, f32, f32),
        radius: f32,
    ) -> Result<(), Self::Error>;

    // 3. 任意路径剪裁 (性能：慢)
    // 对应 tiny-skia 的 clip_path
    // 这是通用兜底方案，任何形状都能裁
    fn push_path_clip(&mut self, path: &Path) -> Result<(), Self::Error>;

    fn pop_clip(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error>;
    fn get_clip_depth(&mut self) -> Result<u32, Self::Error>;

    fn push_transform(&mut self, transform: &Transform2D) -> Result<(), Self::Error>;
    fn pop_transform(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error>;
    fn get_transform_depth(&mut self) -> Result<u32, Self::Error>;

    // ==================== 辅助/输出 ====================
    /// 截取当前渲染目标的内容
    ///
    /// # 参数
    /// - `rect`: 截图区域 `(x, y, width, height)`。
    ///   - x, y: 逻辑坐标起始点
    ///   - width, height: 逻辑尺寸
    ///   - 如果为 `None`，则截取整个当前渲染目标。
    ///
    /// # 返回值
    /// 返回 `Vec<u8>`，通常为 RGBA8 格式的原始像素数据。
    /// 具体数据的步长(Stride)和排列取决于后端实现，但在跨平台层应尽量归一化为 RGBA。
    fn capture_snapshot(
        &mut self,
        rect: Option<(f32, f32, u32, u32)>,
    ) -> Result<Vec<u8>, Self::Error>;
}
