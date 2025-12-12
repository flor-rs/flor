use crate::error::Error;
use crate::render::FlorRender;
use crate::view::view_id::ViewId;
use crate::view::View;
use crate::windows::entry::{WindowEntryVisit, WINDOW_ENTRY_MAP};
use flor_graphics_base::{Color, RenderContext};
use log::trace;
use platform::WindowId;
use std::sync::atomic::Ordering;
use taffy::Layout;

impl View for WindowId {
    fn view_id(&self) -> ViewId {
        if let Some(id) = WINDOW_ENTRY_MAP.get(self) {
            id.value().view_id
        } else {
            panic!("View ID missing");
        }
    }

    fn on_draw(&mut self, render: &mut FlorRender, _layout: Layout) -> Result<(), Error> {
        trace!("window draw");
        // render.create_surface(2, 2)?;

        render.clear(Color::from_hex_str("FFF")?)?;
        let mut text_format = render.create_text_format("宋体")?;

        // 原有的测试绘制保持不变
        let brush = render.create_solid_color_brush(Color::from_hex_str("00ccff")?, None)?;
        render.draw_text("window", &mut text_format, 0., 0., 200., 20., &brush, None)?;

        // --- FPS 绘制逻辑 ---
        let fps = match self.entry().map(|e| e.fps.load(Ordering::Acquire)) {
            None => "None".into(),
            Some(fps) => fps.to_string(),
        };

        // 1. 定义 FPS 文本区域的大小和边距
        let fps_box_width = 100.0; // 预留足够的宽度
        let fps_box_height = 30.0;
        let margin_right = 10.0;
        let margin_top = 5.0;

        // 2. 计算右上角坐标：窗口宽度 - 文本框宽度 - 右边距
        let fps_x = _layout.size.width - fps_box_width - margin_right;
        let fps_y = margin_top;

        // 3. 创建一个新的笔刷，使用更好看的颜色 (深橙红色 #FF4500，醒目且专业)
        let fps_brush = render.create_solid_color_brush(Color::from_hex_str("FF4500")?, None)?;

        // 4. 绘制
        render.draw_text(
            &format!("FPS: {}", fps),
            &mut text_format,
            fps_x,
            fps_y,
            fps_box_width,
            fps_box_height,
            &fps_brush,
            None,
        )?;

        Ok(())
    }
}
