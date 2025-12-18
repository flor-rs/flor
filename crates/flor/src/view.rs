pub mod control_state;
pub mod draw_state;
pub mod focus_manager;
pub mod style;
pub mod view_builder;
pub mod view_id;
pub mod view_state;
pub mod view_storage;

use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::min_wait_time::MinWaitTime;
#[cfg(feature = "svg")]
use crate::render::FlorSvgHandle;
use crate::render::{FlorImageHandle, FlorRender, FlorRenderError, LoadRenderResource};
use crate::view::style::layout::CalcTaffyStyle;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus::render_from_view_id;
use flor_graphics_base::RenderContext;
use flor_platform_base::{InputEvent, KeyCode, KeyState, MousePosition};
use log::trace;
use std::any::Any;
use std::time::{Duration, Instant};
use taffy::{AvailableSpace, Layout, NodeId, Size, Style, TaffyTree};

/// View特征定义了所有UI组件的基本行为
pub trait View {
    /// 获取视图ID
    fn view_id(&self) -> ViewId;

    fn bus_create(&mut self) -> Result<(), Error> {
        self.on_create()?;

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
            .calc_taffy_style(view_id.control_state());
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
    fn bus_update_layout(&mut self, taffy: &mut TaffyTree<ViewId>) -> Result<(), Error> {
        let view_id = self.view_id();
        // 自身处理
        if let Some(state) = VIEW_STORAGE.states.read().get(view_id) {
            let mut state = state.write();
            if let Some(node_id) = state.node_id {
                state.layout = *taffy.layout(node_id)?; // 这行报错
            }
        }
        // 子节点处理
        let views = VIEW_STORAGE.views.read();
        if let Some(child_view_ids) = VIEW_STORAGE.child_ids.read().get(view_id) {
            for view_id in child_view_ids {
                if let Some(view) = views.get(*view_id) {
                    view.write().bus_update_layout(taffy)?;
                }
            }
        }
        Ok(())
    }

    fn bus_frame(&mut self, now: Instant) -> Result<Option<Duration>, Error> {
        let view_id = self.view_id();
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
        let abs_location = (
            abs_location.0 + layout.location.x,
            abs_location.1 + layout.location.y,
        );
        // 自身处理
        trace!("self_view.draw");
        self.on_draw(render, abs_location, layout)?;
        // 绘制子控件
        if let Some(child_view_ids) = VIEW_STORAGE.child_ids.read().get(view_id) {
            for child_id in child_view_ids {
                if let Some(view) = views.get(*child_id) {
                    view.write().bus_draw(render, abs_location)?;
                }
            }
        }
        Ok(())
    }

    fn bus_wheel_scroll_lines_changed_entry(&mut self, lines: u32) -> Result<(), Error> {
        let view_id = self.view_id();
        self.on_wheel_scroll_lines_changed_entry(lines)
            .error_on_err(format!(
                "on_wheel_scroll_lines_changed_entry {{ view_id: {} }}",
                view_id
            ));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .bus_wheel_scroll_lines_changed_entry(lines)
                .error_on_err(format!(
                    "on_wheel_scroll_lines_changed_entry {{ view_id: {} }}",
                    view_id
                ));
        }
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_hit_test(&self, mouse_position: MousePosition, key_state: KeyState) -> bool {
        let Ok(layout) = self.view_id().layout() else {
            return false;
        };
        // 当前控件布局
        let x = layout.location.x;
        let y = layout.location.y;
        let w = layout.size.width;
        let h = layout.size.height;

        // 鼠标位置
        let mx = mouse_position.x as f32;
        let my = mouse_position.y as f32;

        // 鼠标在不在范围内
        mx >= x && mx < x + w && my >= y && my < y + h
    }

    fn on_create(&mut self) -> Result<(), Error> {
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

    /// 测量
    #[allow(unused_variables)]
    fn measure(
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
    #[allow(unused_variables)]
    fn on_mouse_move(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_mouse_leave(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    // ========================================================================
    // [新增] 鼠标按键事件 (Mouse Button Events)
    // 对应 LButton, RButton, MButton 的 Down, Up, DoubleClick
    // 统一使用 &mut self，因为点击通常伴随状态变更(Focus等)
    // ========================================================================

    // ---- 左键 (Left Button) ----
    #[allow(unused_variables)]
    fn on_l_button_down(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_l_button_up(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_l_button_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_l_button_dbl_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    // ---- 右键 (Right Button) ----
    #[allow(unused_variables)]
    fn on_r_button_down(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_r_button_up(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_r_button_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_r_button_dbl_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }

    // ---- 中键 (Middle Button) ----
    #[allow(unused_variables)]
    fn on_m_button_down(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_m_button_up(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_m_button_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn on_m_button_dbl_click(
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

    fn on_focus_gained(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn on_focus_lost(&mut self) -> Result<(), Error> {
        Ok(())
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
    #[allow(unused_variables)]
    fn on_wheel_scroll_lines_changed_entry(&mut self, lines: u32) -> Result<(), Error> {
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
        raw_bytes: Vec<Vec<u8>>,
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
