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

    fn on_draw(&mut self, render: &mut FlorRender, abs_location: (f32, f32), layout: Layout) -> Result<(), Error> {
        trace!("window draw");

        // 解构绝对坐标，方便后续使用
        let (abs_x, abs_y) = abs_location;

        // 清除背景 (注意：如果这是子 View，通常不应该 clear 全屏，除非它是 Root Window)
        // 如果这是根窗口，保持原样；如果是子控件，这行可能会把别人的绘制盖住。
        render.clear(Color::from_hex_str("FFF")?)?;

        // ⚠️ 性能提示：create_text_format 比较耗时，建议在结构体初始化时创建并缓存，不要每帧创建
        let mut text_format = render.create_text_format("宋体")?;

        // --- 绘制 "window" 标签 ---
        let brush = render.create_solid_color_brush(Color::from_hex_str("00ccff")?, None)?;

        // 【修正点 1】：应用绝对坐标
        // 原来是 (0., 0.)，现在是 (abs_x, abs_y)
        render.draw_text(
            "window",
            &mut text_format,
            abs_x,          // left
            abs_y,          // top
            200.,           // width
            20.,            // height
            &brush,
            None
        )?;

        // --- FPS 绘制逻辑 ---
        let fps = match self.entry().map(|e| e.fps.load(Ordering::Acquire)) {
            None => "None".into(),
            Some(fps) => fps.to_string(),
        };

        // 1. 定义 FPS 文本区域的大小和边距
        let fps_box_width = 100.0;
        let fps_box_height = 30.0;
        let margin_right = 10.0;
        let margin_top = 5.0;

        // 2. 计算相对坐标 (相对于当前控件左上角)
        // 右上角 x = 控件宽度 - 文本框宽度 - 右边距
        let fps_relative_x = layout.size.width - fps_box_width - margin_right;
        let fps_relative_y = margin_top;

        // 3. 创建 FPS 笔刷
        let fps_brush = render.create_solid_color_brush(Color::from_hex_str("FF4500")?, None)?;

        // 4. 绘制 FPS
        // 【修正点 2】：应用绝对坐标 (abs_x + relative_x)
        render.draw_text(
            &format!("FPS: {}", fps),
            &mut text_format,
            abs_x + fps_relative_x, // 绝对 X
            abs_y + fps_relative_y, // 绝对 Y
            fps_box_width,
            fps_box_height,
            &fps_brush,
            None,
        )?;

        Ok(())
    }
}

pub trait TryViewId {
    fn try_view_id(&self) -> Option<ViewId>;
}

impl TryViewId for WindowId {
    fn try_view_id(&self) -> Option<ViewId> {
        Some(WINDOW_ENTRY_MAP.get(self)?.value().view_id)
    }
}
