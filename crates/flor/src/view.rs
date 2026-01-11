pub mod class;
pub mod control_state;
pub mod focus_manager;
pub mod handler;
pub mod state_selector;
pub mod view_builder;
pub mod view_id;
pub mod view_state;
pub mod view_storage;
pub mod visual_overflow;
pub mod frame_policy;
pub mod scroll_state;

use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::min_wait_time::MinWaitTime;
#[cfg(feature = "svg")]
use crate::render::FlorSvgHandle;
use crate::render::{FlorImageHandle, FlorRender, FlorRenderError, LoadRenderResource};
use crate::view::control_state::ControlState;
use crate::view::frame_policy::FramePolicy;
use crate::view::state_selector::CalcTaffyStyle;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::visual_overflow::VisualOverflow;
use crate::windows::bus::render_from_view_id;
use flor_graphics_base::RenderContext;
#[cfg(feature = "drag-drop")]
use flor_platform_base::{DragData, DragFormat, DropEffect};
use flor_platform_base::{InputEvent, KeyCode, KeyState, MousePosition, ScrollAxis};
use log::{debug, trace};
use std::any::Any;
use std::time::{Duration, Instant};
use taffy::{AvailableSpace, Display, Layout, NodeId, Size, Style, TaffyTree};

/// View特征定义了所有UI组件的基本行为
pub trait View {
    /// 获取视图ID
    fn view_id(&self) -> ViewId;

    fn bus_create(&mut self) -> Result<(), Error> {
        self.call_create()?;

        if let Some(child_view_ids) = VIEW_STORAGE.child_ids.read().get(self.view_id()) {
            for child_id in child_view_ids {
                if let Some(view) = VIEW_STORAGE.views.read().get(*child_id) {
                    view.write().bus_create()?;
                }
            }
        }
        Ok(())
    }

    /// 创建布局节点
    fn bus_layout_node(&mut self, taffy: &mut TaffyTree<ViewId>) -> Result<NodeId, Error> {
        let view_id = self.view_id();

        let states = VIEW_STORAGE.states.read();
        let Some(view_state_cell) = states.get(view_id) else {
            panic!("View storage's states not found view_id:{view_id:?}");
        };

        // 先读
        let view_state = view_state_cell.read();
        let old_node_id = view_state.node_id;

        let style = view_state
            .layout_style
            .calc_update_taffy_style(view_id.control_state());
        drop(view_state);

        let children = collect_layout_children(view_id, taffy)?;

        let node_id = match (old_node_id, style) {
            (Some(node_id), None) => {
                if !children.is_empty() {
                    taffy.set_children(node_id, &children)?;
                }
                node_id
            }
            (Some(node_id), Some(new_style)) => {
                taffy.set_style(node_id, new_style)?;
                if !children.is_empty() {
                    taffy.set_children(node_id, &children)?;
                }
                node_id
            }
            (None, Some(style)) => {
                if children.is_empty() {
                    taffy.new_leaf_with_context(style, view_id)?
                } else {
                    taffy.new_with_children(style, &children)?
                }
            }
            (None, None) => {
                unreachable!("style is None but node_id is None")
            }
        };

        let mut view_state = view_state_cell.write();
        view_state.node_id = Some(node_id);

        Ok(node_id)
    }

    /// 更新布局
    fn bus_update_layout(
        &mut self,
        taffy: &mut TaffyTree<ViewId>,
        parent_abs_location: (f32, f32),
    ) -> Result<(), Error> {
        let view_id = self.view_id();

        // 计算当前控件的绝对位置
        let current_abs_location: (f32, f32);

        // 自身处理
        if let Some(state) = VIEW_STORAGE.states.read().get(view_id) {
            let mut state = state.write();
            if let Some(node_id) = state.node_id {
                state.layout = *taffy.layout(node_id)?;
            }
            // 计算绝对位置 = 父级绝对位置 + 自身相对位置
            current_abs_location = (
                parent_abs_location.0 + state.layout.location.x,
                parent_abs_location.1 + state.layout.location.y,
            );
            state.abs_location = current_abs_location;
        } else {
            current_abs_location = parent_abs_location;
        }

        // 子节点处理
        let views = VIEW_STORAGE.views.read();
        if let Some(child_view_ids) = VIEW_STORAGE.child_ids.read().get(view_id) {
            for view_id in child_view_ids {
                if let Some(view) = views.get(*view_id) {
                    view.write()
                        .bus_update_layout(taffy, current_abs_location)?;
                }
            }
        }
        Ok(())
    }

    fn bus_frame(&mut self, now: Instant) -> Result<Option<Duration>, Error> {
        let view_id = self.view_id();
        if view_id.calc_current_style()?.display == Display::None {
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

    fn bus_draw(&mut self, render: &mut FlorRender, abs_location: (f32, f32)) -> Result<(), Error> {
        let view_id = self.view_id();
        let views = VIEW_STORAGE.views.read();

        let layout = view_id.layout()?;
        if view_id.calc_current_style()?.display == Display::None {
            return Ok(());
        }

        let abs_location = (
            abs_location.0 + layout.location.x,
            abs_location.1 + layout.location.y,
        );
        // 自身处理
        trace!("self_view.draw");
        let transform_depth = render.get_transform_depth()?;
        let clip_depth = render.get_clip_depth()?;
        self.on_draw(render, abs_location, layout)?;
        // 绘制子控件
        if let Some(child_view_ids) = VIEW_STORAGE.child_ids.read().get(view_id) {
            for child_id in child_view_ids {
                if let Some(view) = views.get(*child_id) {
                    view.write().bus_draw(render, abs_location)?;
                }
            }
        }
        render.pop_clip(Some(clip_depth))?;
        render.pop_transform(Some(transform_depth))?;
        Ok(())
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

    #[allow(unused_variables)]
    fn on_hit_test(&self, mouse_position: MousePosition, key_state: KeyState) -> bool {
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
            return false;
        };

        // 鼠标位置
        let mx = mouse_position.x as f32;
        let my = mouse_position.y as f32;

        debug!(
            "[view({})] on_hit_test: abs_x: {}, abs_y: {}, w: {}, h: {}, mx: {}, my: {}",
            view_id, abs_x, abs_y, w, h, mx, my
        );

        // 鼠标在不在范围内（使用绝对位置）
        mx >= abs_x && mx < abs_x + w && my >= abs_y && my < abs_y + h
    }

    fn on_create(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn call_create(&mut self) -> Result<(), Error> {
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
        layout: Layout,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_draw_overlay(
        &mut self,
        render: &mut FlorRender,
        abs_location: (f32, f32),
        layout: Layout,
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

    // 应该传递父级信息，到这里与自己比较
    fn on_visual_test_entry(&self, test_bounds: (f32, f32, f32, f32)) -> bool {
        // 1. 获取自身的视觉包围盒 (已包含阴影扩散)
        let (my_x, my_y, my_w, my_h) = self.visual_rect();
        let (test_x, test_y, test_w, test_h) = test_bounds;

        // 2. 基础有效性检查 (如果任意一方宽高非正，视为不可见)
        // 注意：VisualOverflow 可能让原本 w=0 的控件变大，所以检查 my_w 而不是 layout w
        if my_w <= 0.0 || my_h <= 0.0 || test_w <= 0.0 || test_h <= 0.0 {
            return false;
        }

        // 3. AABB 相交测试 (只要四个方向有一个方向错开了，就是不相交)
        // 逻辑：(我左 > 你右) 或 (我右 < 你左) 或 (我顶 > 你底) 或 (我底 < 你顶) => 不相交
        let is_disjoint = my_x >= test_x + test_w       // 我的左边 在 你的右边 之外
                || my_x + my_w <= test_x         // 我的右边 在 你的左边 之外
                || my_y >= test_y + test_h       // 我的顶边 在 你的底边 之外
            || my_y + my_h <= test_y; // 我的底边 在 你的顶边 之外

        !is_disjoint
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

pub(crate) fn collect_layout_children(
    parent_id: ViewId,
    taffy: &mut TaffyTree<ViewId>,
) -> Result<Vec<NodeId>, Error> {
    // 1. 读锁获取子节点列表
    // 注意：这里用代码块限制锁的范围，防止带锁进行后续递归
    let child_list: Option<Vec<ViewId>> = { VIEW_STORAGE.child_ids.read().get(parent_id).cloned() };

    if let Some(childs) = child_list {
        let mut node_ids = Vec::with_capacity(childs.len());
        for child_view_id in childs {
            if let Some(dyn_view) = VIEW_STORAGE.views.read().get(child_view_id) {
                // 递归调用
                let node_id = dyn_view.write().bus_layout_node(taffy)?;
                node_ids.push(node_id);
            }
        }
        Ok(node_ids)
    } else {
        Ok(Vec::new())
    }
}
