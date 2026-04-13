use crate::error::Error;
use crate::render::FlorRenderer;
use crate::view::ViewId;
use crate::view::{ControlState, View};
use crate::windows::{WindowEntryVisit, WINDOW_ENTRY_MAP};
use flor_base::graphics::RenderContext;
use flor_base::platform::WindowApi;
use flor_base::types::Color;
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

    fn on_draw(
        &mut self,
        render: &mut FlorRenderer,
        _control_state: ControlState,
        _abs_location: (f32, f32),
        _layout: Layout,
    ) -> Result<(), Error> {
        trace!("window draw");

        let Some(entry) = self.entry() else {
            return Ok(());
        };
        render.clear(entry.background_color)?;
        if !entry.show_fps {
            return Ok(());
        }

        // l,t: i32  w,h: u32
        let (w, _) = self.get_client_size()?;

        // --- FPS ---
        let mut text_format = render.create_text_format("")?;

        let fps = {
            let fps = entry.fps.load(Ordering::Acquire);
            if fps < 0 {
                "-".to_string()
            } else {
                fps.to_string()
            }
        };

        // 固定参数
        let fps_box_width: f32 = 100.0;
        let margin_right: f32 = 10.0;
        let margin_top: f32 = 20.0;

        // 3. 计算 X：窗口宽度 - 盒子宽度 - 右边距
        // 如果 FPS 依然看不见，试着减小 w 的值看它是否从右侧滑入
        let fps_x = w as f32 - fps_box_width - margin_right;
        let fps_y = margin_top;

        let fps_brush = render.create_solid_color_brush(Color::from_hex_str("FF4500")?, None)?;

        render.draw_text(
            &format!("FPS: {}", fps),
            &mut text_format,
            fps_x,
            fps_y,
            fps_box_width,
            30.0,
            &fps_brush,
            None,
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
