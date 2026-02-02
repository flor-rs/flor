pub mod class;
pub mod control_state;
pub mod focus_manager;
pub mod frame_policy;
pub mod handler;
pub mod scroll_state;
pub mod view_builder;
pub mod view_id;
pub mod view_state;
pub mod view_storage;
pub mod visual_overflow;
pub mod resolver;

use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::min_wait_time::MinWaitTime;
#[cfg(feature = "svg")]
use crate::render::FlorSvgHandle;
use crate::render::{FlorImageHandle, FlorRender, FlorRenderError, LoadRenderResource};
use crate::view::control_state::ControlState;
use crate::view::frame_policy::FramePolicy;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::visual_overflow::VisualOverflow;
use crate::windows::bus::render_from_view_id;
use crate::ComputedLayout;
use flor_base::graphics::RenderContext;
#[cfg(feature = "drag-drop")]
use flor_base::platform::{DragData, DragFormat, DropEffect};
use flor_base::platform::{InputEvent, KeyCode, KeyState, MousePosition, ScrollAxis};
use std::any::Any;
use std::time::{Duration, Instant};
use taffy::{AvailableSpace, Display, Size, Style};

/// View特征定义了所有UI组件的基本行为
pub trait View {
    /// 获取视图ID
    fn view_id(&self) -> ViewId;

    fn tag(&self) -> &str {
        "View"
    }

    fn bus_frame(&mut self, now: Instant) -> Result<Option<Duration>, Error> {
        let view_id = self.view_id();
        if view_id.with_current_style(|style| style.display == Display::None)? {
            return Ok(None);
        }
        if !view_id.visual() {
            return Ok(None);
        }
        let views = VIEW_STORAGE.views.read();
        let mut min_wait_time = self.on_frame(now)?;
        // 绘制子控件
        if let Some(child_view_ids) = VIEW_STORAGE.child_ids.read().get(view_id) {
            for child_id in child_view_ids {
                if let Some(view) = views.get(*child_id) {
                    let child_wait_time = view.write().bus_frame(now)?;
                    min_wait_time.update_to_min_wait_time(child_wait_time);
                }
            }
        }
        Ok(min_wait_time)
    }

    fn bus_wheel_scroll_lines_changed(
        &mut self,
        axis: ScrollAxis,
        delta: f32,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        let view_id = self.view_id();
        self.on_wheel_scroll_lines_changed(axis, delta, key_state, mouse_position)
            .error_on_err(format!(
                "on_wheel_scroll_lines_changed {{ view_id: {} }}",
                view_id
            ));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .bus_wheel_scroll_lines_changed(axis, delta, key_state, mouse_position)
                .error_on_err(format!(
                    "on_wheel_scroll_lines_changed {{ view_id: {} }}",
                    view_id
                ));
        }
        Ok(())
    }

    /// 判定：鼠标是否在内容区域（不包含滚动条）
    ///
    /// 注意：mouse_position 是控件局部坐标（0,0 = 控件左上角）
    fn on_hit_test(&self, mouse_position: MousePosition, _key_state: KeyState) -> bool {
        let view_id = self.view_id();

        // 获取控件尺寸
        let Ok((w, h)) = view_id.with_state(|state| {
            (
                state.layout.size.width,
                state.layout.size.height,
            )
        }) else {
            return false;
        };

        let mx = mouse_position.x as f32;
        let my = mouse_position.y as f32;

        // 检查是否在内容矩形内（局部坐标系）
        // [0, w) 和 [0, h)
        mx >= 0.0 && mx < w && my >= 0.0 && my < h
    }

    /// 判定：鼠标是否在 Overlay 区域（滚动条、调整手柄等）
    /// 注意：这个函数在 Hit Test 流程中优先级最高
    ///
    /// mouse_position 是控件局部坐标（0,0 = 控件左上角）
    fn on_hit_test_overlay(&self, mouse_position: MousePosition, _key_state: KeyState) -> bool {
        let view_id = self.view_id();

        // 需要尺寸信息（不再需要 abs_location）
        let Ok((w, h, sb_w, sb_h)) = view_id.with_state(|state| {
            (
                state.layout.size.width,
                state.layout.size.height,
                state.layout.scrollbar_size.width,
                state.layout.scrollbar_size.height,
            )
        }) else {
            return false;
        };

        // 如果没有滚动条，直接返回 false
        if sb_w <= 0.0 && sb_h <= 0.0 {
            return false;
        }

        let mx = mouse_position.x as f32;
        let my = mouse_position.y as f32;

        // 边界定义（局部坐标系，原点在控件左上角）
        let right_edge = w;           // 内容右边界
        let bottom_edge = h;          // 内容下边界
        let total_w = w + sb_w;       // 总宽度（含右侧滚动条）
        let total_h = h + sb_h;       // 总高度（含底部滚动条）

        // 1. 检查垂直滚动条区域 (位于右侧)
        // 区域：X 在 [w, w + sb_w), Y 在 [0, total_h)
        if sb_w > 0.0 {
            if mx >= right_edge && mx < total_w &&
                my >= 0.0 && my < total_h {
                return true;
            }
        }

        // 2. 检查水平滚动条区域 (位于底部)
        // 区域：Y 在 [h, h + sb_h), X 在 [0, total_w)
        if sb_h > 0.0 {
            if my >= bottom_edge && my < total_h &&
                mx >= 0.0 && mx < total_w {
                return true;
            }
        }

        // 补充说明：
        // 上述逻辑中，右下角的交汇处 (Corner) 无论是被判定为垂直还是水平滚动条的一部分，
        // 都会返回 true，这符合 Overlay 拦截的预期。

        false
    }

    fn on_create(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn call_create(&mut self) -> Result<(), Error> {
        VIEW_STORAGE.active_pending_effect_id(self.view_id());
        self.on_create()?;
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_create_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id());
        }
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_update_state(&mut self, state: Box<dyn Any>) {}

    #[allow(unused_variables)]
    fn on_frame(&mut self, now: Instant) -> Result<Option<Duration>, Error> {
        Ok(None)
    }

    /// 重绘视图
    #[allow(unused_variables)]
    fn on_draw(
        &mut self,
        render: &mut FlorRender,
        abs_location: (f32, f32),
        layout: ComputedLayout,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_draw_overlay(
        &mut self,
        render: &mut FlorRender,
        abs_location: (f32, f32),
        layout: ComputedLayout,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// 测量
    #[allow(unused_variables)]
    fn on_measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        style: &Style,
        control_state: ControlState,
        render: &mut FlorRender,
    ) -> Result<Size<f32>, Error> {
        Ok(Size::ZERO)
    }

    #[allow(unused_variables)]
    fn on_mouse_enter(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call_mouse_enter(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_mouse_enter(key_state, mouse_position)
            .error_on_err(format!(
                "on_mouse_enter {{ key_state: {:?}, mouse_position: {:?} }}",
                key_state, mouse_position
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_mouse_enter_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id());
        }
    }
    #[allow(unused_variables)]
    fn on_mouse_move(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call_mouse_move(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_mouse_move(key_state, mouse_position)
            .error_on_err(format!(
                "on_mouse_move {{ key_state: {:?}, mouse_position: {:?} }}",
                key_state, mouse_position
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_mouse_move_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }
    #[allow(unused_variables)]
    fn on_mouse_leave(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call_mouse_leave(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_mouse_leave(key_state, mouse_position)
            .error_on_err(format!(
                "on_mouse_leave {{ key_state: {:?}, mouse_position: {:?} }}",
                key_state, mouse_position
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_mouse_leave_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id());
        }
    }

    // ========================================================================
    // [新增] 鼠标按键事件 (Mouse Button Events)
    // 对应 LButton, RButton, MButton 的 Down, Up, DoubleClick
    // 统一使用 &mut self，因为点击通常伴随状态变更(Focus等)
    // ========================================================================

    // ---- 左键 (Left Button) ----
    fn call_button_down(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_button_down(key_state, mouse_position)
            .error_on_err(format!("on_button_down {{ view_id:{} }}", self.view_id()));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_button_down_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_button_down(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn call_button_up(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_button_up(key_state, mouse_position)
            .error_on_err(format!("on_button_up {{ view_id:{} }}", self.view_id()));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_button_up_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_button_up(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn call_click(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_click(key_state, mouse_position)
            .error_on_err(format!("on_click {{ view_id:{} }}", self.view_id()));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_click_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call_double_click(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_double_click(key_state, mouse_position)
            .error_on_err(format!("on_double_click {{ view_id:{} }}", self.view_id()));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_double_click_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_double_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    // ---- 右键 (Right Button) ----
    fn call_right_button_down(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_right_button_down(key_state, mouse_position)
            .error_on_err(format!(
                "on_right_button_down {{ view_id:{} }}",
                self.view_id()
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_right_button_down_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_right_button_down(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn call_right_button_up(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_right_button_up(key_state, mouse_position)
            .error_on_err(format!(
                "on_right_button_up {{ view_id:{} }}",
                self.view_id()
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_right_button_up_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_right_button_up(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn call_right_button_click(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_right_button_click(key_state, mouse_position)
            .error_on_err(format!(
                "on_right_button_click {{ view_id:{} }}",
                self.view_id()
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_right_button_click_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_right_button_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call_right_button_double_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) {
        self.on_right_button_double_click(key_state, mouse_position)
            .error_on_err(format!(
                "on_right_button_double_click {{ view_id:{} }}",
                self.view_id()
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_right_button_double_click_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_right_button_double_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    // ---- 中键 (Middle Button) ----
    fn call_middle_button_down(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_middle_button_down(key_state, mouse_position)
            .error_on_err(format!(
                "on_middle_button_down {{ view_id:{} }}",
                self.view_id()
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_middle_button_down_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_middle_button_down(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn call_middle_button_up(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_middle_button_up(key_state, mouse_position)
            .error_on_err(format!(
                "on_middle_button_up {{ view_id:{} }}",
                self.view_id()
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_middle_button_up_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_middle_button_up(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn call_middle_button_click(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        self.on_middle_button_click(key_state, mouse_position)
            .error_on_err(format!(
                "on_middle_button_click {{ view_id:{} }}",
                self.view_id()
            ));
        // 注意：ViewHandler 中似乎没有 on_middle_button_click_handler
        // 但为了保持一致性，如果确实没有，这里就不调用 handler
        // 检查 handler.rs: 确实没有 on_middle_button_click_handler
    }

    #[allow(unused_variables)]
    fn on_middle_button_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn call_middle_button_double_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) {
        self.on_middle_button_double_click(key_state, mouse_position)
            .error_on_err(format!(
                "on_middle_button_double_click {{ view_id:{} }}",
                self.view_id()
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_middle_button_double_click_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), key_state, mouse_position);
        }
    }

    #[allow(unused_variables)]
    fn on_middle_button_double_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_key_down(
        &mut self,
        code: KeyCode,
        is_alt: bool,
        is_ctrl: bool,
        is_shift: bool,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call_key_down(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {
        self.on_key_down(code, is_alt, is_ctrl, is_shift)
            .error_on_err(format!(
                "on_key_down {{ code: {:?}, is_alt: {:?}, is_ctrl: {:?}, is_shift: {:?} }}",
                code, is_alt, is_ctrl, is_shift
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_key_down_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), code, is_alt, is_ctrl, is_shift);
        }
    }

    #[allow(unused_variables)]
    fn on_key_up(
        &mut self,
        code: KeyCode,
        is_alt: bool,
        is_ctrl: bool,
        is_shift: bool,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call_key_up(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {
        self.on_key_up(code, is_alt, is_ctrl, is_shift)
            .error_on_err(format!(
                "on_key_up {{ code: {:?}, is_alt: {:?}, is_ctrl: {:?}, is_shift: {:?} }}",
                code, is_alt, is_ctrl, is_shift
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_key_up_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), code, is_alt, is_ctrl, is_shift);
        }
    }

    fn on_focus_gained(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn call_focus_gained(&mut self) {
        self.on_focus_gained()
            .error_on_err(format!("on_focus_gained {{ view_id:{} }}", self.view_id()));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_focus_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id());
        }
    }
    fn on_focus_lost(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn call_focus_lost(&mut self) {
        self.on_focus_lost()
            .error_on_err(format!("on_focus_lost {{ view_id:{} }}", self.view_id()));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_blur_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id());
        }
    }

    fn on_ime_start(&mut self) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_ime_input(&mut self, input_event: &InputEvent) -> Result<(), Error> {
        Ok(())
    }

    fn on_ime_end(&mut self) -> Result<(), Error> {
        Ok(())
    }

    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    fn on_drag_enter(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> Result<DropEffect, Error> {
        Ok(DropEffect::None)
    }

    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    fn call_drag_enter(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> DropEffect {
        let mut effect = self
            .on_drag_enter(key_state, mouse_position, format)
            .log_err(format!("on_drag_enter {{ view_id:{} }}", self.view_id()))
            .unwrap_or(DropEffect::None);

        if let Some(handler_lock) = VIEW_STORAGE.handlers.read().get(self.view_id()) {
            let handler = handler_lock.read();
            if let Some(h) = &handler.on_drag_enter_handler {
                h.0(
                    self.view_id(),
                    key_state,
                    mouse_position,
                    format,
                    &mut effect,
                );
            }
        }
        effect
    }

    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    fn on_drag_over(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> Result<DropEffect, Error> {
        Ok(DropEffect::None)
    }

    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    fn call_drag_over(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> DropEffect {
        let mut effect = self
            .on_drag_over(key_state, mouse_position, format)
            .log_err(format!("on_drag_over {{ view_id:{} }}", self.view_id()))
            .unwrap_or(DropEffect::None);

        if let Some(handler_lock) = VIEW_STORAGE.handlers.read().get(self.view_id()) {
            let handler = handler_lock.read();

            if let Some(h) = &handler.on_drag_over_handler {
                h.0(
                    self.view_id(),
                    key_state,
                    mouse_position,
                    format,
                    &mut effect,
                );
            }
        }
        effect
    }

    #[cfg(feature = "drag-drop")]
    fn on_drag_leave(&mut self) -> Result<(), Error> {
        Ok(())
    }

    #[cfg(feature = "drag-drop")]
    #[cfg(feature = "drag-drop")]
    fn call_drag_leave(&mut self) {
        self.on_drag_leave()
            .error_on_err(format!("on_drag_leave {{ view_id:{} }}", self.view_id()));
        if let Some(handler_lock) = VIEW_STORAGE.handlers.read().get(self.view_id()) {
            let handler = handler_lock.read();
            if let Some(h) = &handler.on_drag_leave_handler {
                h.0(self.view_id());
            }
        }
    }

    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    fn on_drop(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        data: &DragData,
    ) -> Result<DropEffect, Error> {
        Ok(DropEffect::None)
    }

    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    #[cfg(feature = "drag-drop")]
    #[allow(unused_variables)]
    fn call_drop(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        data: &DragData,
    ) -> DropEffect {
        let mut effect = self
            .on_drop(key_state, mouse_position, data)
            .log_err(format!("on_drop {{ view_id:{} }}", self.view_id()))
            .unwrap_or(DropEffect::None);

        if let Some(handler_lock) = VIEW_STORAGE.handlers.read().get(self.view_id()) {
            let handler = handler_lock.read();

            if let Some(h) = &handler.on_drop_handler {
                h.0(self.view_id(), key_state, mouse_position, data, &mut effect);
            }
        }
        effect
    }

    #[allow(unused_variables)]
    fn on_wheel_scroll_lines_changed(
        &mut self,
        axis: ScrollAxis,
        delta: f32,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call_wheel_scroll_lines_changed(
        &mut self,
        axis: ScrollAxis,
        delta: f32,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) {
        self.on_wheel_scroll_lines_changed(axis, delta, key_state, mouse_position)
            .error_on_err(format!(
                "on_wheel_scroll_lines_changed {{ view_id: {} }}",
                self.view_id()
            ));
        let handler = VIEW_STORAGE
            .handlers
            .read()
            .get(self.view_id())
            .and_then(|h| h.read().on_wheel_settings_changed_handler.clone());
        if let Some(h) = handler {
            h.0(self.view_id(), axis, delta, key_state, mouse_position);
        }
    }

    /// 如果手动重写且需要更新里面的数据，请请求布局重新计算
    fn visual_rect(&self) -> (f32, f32, f32, f32) {
        let view_id = self.view_id();

        // 获取控件的布局信息和绝对位置
        let Ok((abs_x, abs_y, w, h)) = view_id.with_state(|state| {
            (
                state.abs_location.0,
                state.abs_location.1,
                state.layout.size.width,
                state.layout.size.height,
            )
        }) else {
            return (0f32, 0f32, 0f32, 0f32);
        };

        match self.on_visual_overflow() {
            VisualOverflow::None => (abs_x, abs_y, w, h),
            // 统一扩散：x,y 减去 v，宽高各增加 2*v
            VisualOverflow::Uniform(v) => (abs_x - v, abs_y - v, w + v * 2.0, h + v * 2.0),
            // 自定义扩散：x减左，y减上，宽加左右，高加上下
            VisualOverflow::Custom {
                left,
                top,
                right,
                bottom,
            } => (
                abs_x - left,
                abs_y - top,
                w + left + right,
                h + top + bottom,
            ),
            VisualOverflow::Path(path) => {
                let (x, y, w, h) = path.get_bounds();
                (abs_x + x, abs_y + y, w, h)
            }
        }
    }

    fn on_visual_overflow(&self) -> VisualOverflow {
        VisualOverflow::None
    }

    fn on_frame_policy(&self) -> FramePolicy {
        FramePolicy::VisibleOnly
    }

    fn on_child_push(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn on_child_dispose(&mut self) -> Result<(), Error> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_update_class(&mut self, control_state: ControlState, class: &str) -> Result<(), Error> {
        Ok(())
    }
}

impl<T: View> LoadRenderResource for T {
    fn load_image(&self, image: &[u8]) -> Result<FlorImageHandle, FlorRenderError> {
        if let Some(x) = render_from_view_id(self.view_id()) {
            let mut render = x.write();
            render.create_image_from_bytes(&image)
        } else {
            Err(FlorRenderError::RenderNotFound)
        }
    }

    fn load_raw_image(
        &self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<FlorImageHandle, FlorRenderError> {
        if let Some(x) = render_from_view_id(self.view_id()) {
            let mut render = x.write();
            render.create_image_from_raw_bytes(raw_bytes, width, height, delays)
        } else {
            Err(FlorRenderError::RenderNotFound)
        }
    }

    #[cfg(feature = "svg")]
    fn load_svg(&self, svg: &[u8]) -> Result<FlorSvgHandle, FlorRenderError> {
        if let Some(x) = render_from_view_id(self.view_id()) {
            let mut render = x.write();
            render.create_svg(svg)
        } else {
            Err(FlorRenderError::RenderNotFound)
        }
    }
}
