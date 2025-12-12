use crate::error::Error;
use crate::log_error::LogError;
use crate::render::FlorRender;
use crate::view::style::layout::CalcTaffyStyle;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use crate::windows::entry::WindowEntryVisit;
use flor_graphics_base::RenderContext;
use flor_platform_base::{MousePosition, WindowOperations};
use flor_platform_base::{KeyCode, KeyState};
use log::{trace, warn};
use platform::WindowId;
use slotmap::Key;
use std::ops::DerefMut;
use std::time::{Duration, Instant};
use taffy::{AvailableSpace, Size, Style};
use crate::windows::bus::{render, render_from_view_id};

/// 总线的事件分发入口，给窗口用的
pub trait WindowBusDispatchEntry {
    fn create_entry(&mut self) -> Result<(), Error>;
    fn bus_refresh_layout_entry(&self) -> Result<(), Error>;
    fn bus_re_draw_entry(&self) -> Result<(), Error>;
    fn bus_frame_entry(&self) -> Result<Option<Duration>, Error>;
    fn bus_mouse_move_entry(&self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_key_down_entry(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool);
    fn bus_key_up_entry(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool);
}

impl WindowBusDispatchEntry for WindowId {
    fn create_entry(&mut self) -> Result<(), Error> {
        let view_id = self.view_id();
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write().bus_create()?;
        }
        Ok(())
    }

    // todo 脏布局检测
    fn bus_refresh_layout_entry(&self) -> Result<(), Error> {
        trace!("enter relayout");

        let Some(entry_ref) = self.entry() else {
            warn!("window not found entry in re_layout_entry function.");
            return Ok(());
        };
        let layout_tree = &mut entry_ref.taffy_tree.write();
        let view_id = entry_ref.view_id;

        trace!("let states = VIEW_STORAGE.states.read()");
        let states = VIEW_STORAGE.states.read();
        let Some(view_state) = states.get(view_id) else {
            panic!("View storage's states not found view_id:{view_id:?}");
        };
        let self_style = view_state
            .read()
            .layout_style
            .calc_taffy_style(view_id.control_state());

        let root_node_id = if let Some(childs) = { VIEW_STORAGE.child_ids.read().get(view_id) } {
            let mut node_ids = Vec::new();
            for child_view_id in childs {
                if let Some(dyn_view) = { VIEW_STORAGE.views.read().get(*child_view_id) } {
                    node_ids.push(dyn_view.write().bus_layout_node(layout_tree)?);
                }
            }
            layout_tree.new_with_children(
                Style {
                    size: Size::from_percent(1.0, 1.0),
                    ..self_style.clone()
                },
                &node_ids,
            )?
        } else {
            layout_tree.new_leaf(self_style.clone())?
        };

        // 更新node_id
        if let Some(view_state) = VIEW_STORAGE.states.read().get(view_id) {
            let mut view_state = view_state.write();
            if let Some(old_node_id) = view_state.node_id.take() {
                layout_tree.remove(old_node_id)?;
            }
            view_state.node_id = Some(root_node_id);
        }

        let client_size = self.get_client_size()?;
        layout_tree.compute_layout_with_measure(
            root_node_id,
            Size {
                height: AvailableSpace::Definite(client_size.1 as f32),
                width: AvailableSpace::Definite(client_size.0 as f32),
            },
            |known_dimensions, available_space, _node_id, view_id, style| {
                trace!("view_id: {:?}, available_space: {available_space:?}, known_dimensions: {known_dimensions:?}, style: {style:?}",view_id.as_ref().map(|v|v.data()));
                if let Some(view_id) = view_id {
                    if let Some(dyn_view) = VIEW_STORAGE.views.read().get(*view_id) {
                        trace!("find view");
                        if let Some(render) = render_from_view_id(*view_id).as_deref() {
                            trace!("find view_bus");
                            let mut render = render.write();
                            let render = render.deref_mut();

                            let mut view = dyn_view
                                .write();
                            return
                                view.measure(known_dimensions, available_space, style, render)
                                .unwrap_or(Size::ZERO);
                        }
                    }
                }
                Size::ZERO
            },
        )?;
        // dbg!(self.get_height(),self.get_width());
        // dbg!(self.layout_tree.layout(root_node)?);
        {
            trace!("bus_update_layout begin");
            // 直接从view发起访问
            if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
                view.write().bus_update_layout(layout_tree)?;
            }
            trace!("bus_update_layout end");
        }
        // entry_ref.clear_layout_dirty();
        // self.request_redraw();
        Ok(())
    }

    fn bus_re_draw_entry(&self) -> Result<(), Error> {
        let Some(render) = render(*self) else {
            return Ok(());
        };
        let mut render = render.write();
        render.begin().log_error("fail begin render");
        let view_id = self.view_id();
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .bus_draw(&mut render)
                .log_error(format!("draw({:?}) error", view_id));
        }
        render.end().log_error("fail end render");
        Ok(())
    }

    fn bus_frame_entry(&self) -> Result<Option<Duration>, Error> {
        if let Some(view) = VIEW_STORAGE.views.read().get(self.view_id()) {
            return view.write().bus_frame(Instant::now());
        }
        Ok(None)
    }

    fn bus_mouse_move_entry(&self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self.view_id();
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write().bus_mouse_move(key_state, mouse_position);
        }
    }

    fn bus_key_down_entry(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {
        let views = VIEW_STORAGE.views.read();

        if let Some(view_id) = self
            .entry()
            .and_then(|entry| entry.focus_manager.current_view_id())
        {
            if let Some(view) = views.get(view_id) {
                view.write().on_key_down(code, is_alt, is_ctrl, is_shift);
                return;
            }
        }

        self.on_key_down(code, is_alt, is_ctrl, is_shift);
    }

    fn bus_key_up_entry(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {
        let views = VIEW_STORAGE.views.read();

        if let Some(view_id) = self
            .entry()
            .and_then(|entry| entry.focus_manager.current_view_id())
        {
            if let Some(view) = views.get(view_id) {
                view.write().on_key_up(code, is_alt, is_ctrl, is_shift);
                return;
            }
        }

        self.on_key_up(code, is_alt, is_ctrl, is_shift);
    }
}
