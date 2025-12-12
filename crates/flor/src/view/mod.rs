use crate::error::Error;
use crate::min_wait_time::MinWaitTime;
#[cfg(feature = "svg")]
use crate::render::FlorSvgHandle;
use crate::render::{FlorImageHandle, FlorRender, FlorRenderError, LoadRenderResource};
use crate::view::style::layout::CalcTaffyStyle;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus::render_from_view_id;
use crate::windows::entry::WindowEntryVisit;
use flor_graphics_base::RenderContext;
use flor_platform_base::{KeyCode, KeyState, MousePosition};
use log::trace;
use std::any::Any;
use std::time::{Duration, Instant};
use taffy::{AvailableSpace, Layout, NodeId, Size, Style, TaffyTree};

pub mod button;
pub mod control_state;
pub mod draw_state;
pub mod focus_manager;
pub mod into_box_view;
pub mod label;
pub mod style;
pub mod v_stack;
pub mod view_builder;
pub mod view_event;
pub mod view_id;
pub mod view_state;
pub mod view_storage;

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

        let Some(view_state) = states.get(view_id) else {
            panic!("View storage's states not found view_id:{view_id:?}");
        };
        let style = view_state
            .read()
            .layout_style
            .calc_taffy_style(view_id.control_state());

        let node_id = if let Some(childs) = VIEW_STORAGE.child_ids.read().get(view_id) {
            let mut node_ids = Vec::new();
            for child_view_id in childs {
                if let Some(dyn_view) = { VIEW_STORAGE.views.read().get(*child_view_id) } {
                    node_ids.push((**dyn_view.write()).bus_layout_node(taffy)?);
                }
            }
            taffy.new_with_children(style.clone(), &node_ids)?
        } else {
            taffy.new_leaf_with_context(style.clone(), view_id)?
        };

        // 更新node_id
        if let Some(view_state) = VIEW_STORAGE.states.read().get(view_id) {
            let mut view_state = view_state.write();
            if let Some(old_node_id) = view_state.node_id.take() {
                taffy.remove(old_node_id)?;
            }
            view_state.node_id = Some(node_id);
        }

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

    fn bus_draw(&mut self, render: &mut FlorRender) -> Result<(), Error> {
        let view_id = self.view_id();
        let views = VIEW_STORAGE.views.read();
        let layout = view_id.with_state(|state| state.layout)?;
        // 自身处理
        trace!("self_view.draw");
        self.on_draw(render, layout)?;
        // 绘制子控件
        if let Some(child_view_ids) = VIEW_STORAGE.child_ids.read().get(view_id) {
            for child_id in child_view_ids {
                if let Some(view) = views.get(*child_id) {
                    view.write().bus_draw(render)?;
                }
            }
        }
        Ok(())
    }

    // 总线调度
    fn bus_mouse_move(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self.view_id();
        let views = VIEW_STORAGE.views.read();

        let layout = view_id.layout();
        // 当前控件布局
        let x = layout.location.x;
        let y = layout.location.y;
        let w = layout.size.width;
        let h = layout.size.height;

        // 鼠标位置
        let mx = mouse_position.x as f32;
        let my = mouse_position.y as f32;
        trace!(
            "x: {}, y: {}, w: {}, h: {}, mx: {},my: {}",
            x,
            y,
            w,
            h,
            mx,
            my
        );
        if mx >= x && mx < x + w && my >= y && my < y + h {
            if let Some(win_id) = view_id.window_id() {
                if let Some(mut entry) = win_id.entry_mut() {
                    trace!("set active view_id:{view_id:?}");
                    entry.active_id = Some(view_id);
                }
            }
        }

        self.on_mouse_move(key_state, mouse_position);
        if let Some(child_view_ids) = VIEW_STORAGE.child_ids.read().get(view_id) {
            for child_id in child_view_ids {
                if let Some(view) = views.get(*child_id) {
                    view.write().bus_mouse_move(key_state, mouse_position);
                }
            }
        }
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
    fn on_draw(&mut self, render: &mut FlorRender, layout: Layout) -> Result<(), Error> {
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

    // 鼠标进入，用户方法
    #[allow(unused_variables)]
    fn on_mouse_move(&mut self, key_state: KeyState, mouse_position: MousePosition) {}

    #[allow(unused_variables)]
    fn on_key_down(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {}
    #[allow(unused_variables)]
    fn on_key_up(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {}

    fn on_focus_gained(&mut self) {}
    fn on_focus_lost(&mut self) {}
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

    fn load_raw_image(&self, raw_bytes: Vec<Vec<u8>>, width: u32, height: u32, delays: Vec<u16>) -> Result<FlorImageHandle, FlorRenderError> {
        if let Some(x) = render_from_view_id(self.view_id()) {
            let mut render = x.write();
            render.create_image_from_raw_bytes(raw_bytes,width,height,delays)
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
