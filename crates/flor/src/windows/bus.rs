use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::render::FlorRender;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::{WindowEntryVisit, WINDOW_ENTRY_MAP};
use crate::FlorGui;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use flor_base::graphics::RenderContext;
use flor_base::platform::KeyCode;
use log::trace;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use platform::base::HandleResult;
use platform::base::Message;
use platform::WindowId;

pub static RENDERS: Lazy<DashMap<WindowId, RwLock<FlorRender>>> = Lazy::new(|| Default::default());

/// 注册窗口到总线
#[inline]
pub fn register_render(window_id: WindowId, render: FlorRender) {
    RENDERS.insert(window_id, RwLock::new(render));
}

/// 从总线移除窗口
pub fn remove_window(window_id: WindowId) {
    trace!("Removing window {:?}", window_id);
    WINDOW_ENTRY_MAP.remove(&window_id);
    if let Some(_) = RENDERS.remove(&window_id) {
        if RENDERS.is_empty() {
            trace!("bus is null,set exit.");
            FlorGui.exit();
        } else {
            trace!("bus is not null");
        }
    } else {
        trace!("not found window_id in bus.");
    }
}

#[inline]
pub fn render<'a>(window_id: WindowId) -> Option<Ref<'a, WindowId, RwLock<FlorRender>>> {
    RENDERS.get(&window_id)
}

pub fn render_from_view_id<'a>(view_id: ViewId) -> Option<Ref<'a, WindowId, RwLock<FlorRender>>> {
    let ret = VIEW_STORAGE
        .window_ids
        .read()
        .get(view_id)
        .and_then(|window_id| RENDERS.get(&window_id));
    ret
}

/// 处理窗口事件
///
/// 小心访问bus卡死
///
pub fn event(mut window_id: WindowId, message: Message) -> Result<HandleResult, Error> {
    let handle_result = match message {
        Message::Draw => {
            trace!("Draw begin");
            if !window_id
                .entry()
                .map(|e| e.is_continuous_rendering())
                .unwrap_or(true)
            {
                window_id.bus_re_draw_entry()?;
            }
            HandleResult::Handled
        }
        Message::Resize { width, height } => {
            window_id.bus_refresh_layout_entry()?;
            {
                let bus = render(window_id);
                let Some(render) = bus.as_deref() else {
                    return Ok(HandleResult::Default);
                };
                let mut render = render.write();
                render
                    .update_window_size(width, height)
                    .error_on_err("fail update size");
            }
            if window_id
                .entry()
                .map(|e| e.is_continuous_rendering())
                .unwrap_or(true)
            {
                window_id.bus_re_draw_entry().error_on_err("fail draw");
            }
            HandleResult::Handled
        }
        Message::ImeStart => {
            window_id.bus_ime_start_entry();
            HandleResult::Handled
        }
        Message::ImeInput(input_event) => {
            window_id.bus_ime_input_entry(input_event);
            HandleResult::Handled
        }
        Message::ImeEnd => {
            window_id.bus_ime_end_entry();
            HandleResult::Handled
        }
        // ==================== 左键 (Left Button) ====================
        Message::LButtonDown {
            key_state,
            mouse_position,
        } => {
            window_id.bus_button_down_entry(key_state, mouse_position);
            HandleResult::Handled
        }
        Message::LButtonUp {
            key_state,
            mouse_position,
        } => {
            window_id.bus_button_up_entry(key_state, mouse_position);
            HandleResult::Handled
        }
        Message::LButtonDoubleClick {
            key_state,
            mouse_position,
        } => {
            window_id.bus_double_click_entry(key_state, mouse_position);
            HandleResult::Handled
        }

        // ==================== 右键 (Right Button) ====================
        Message::RButtonDown {
            key_state,
            mouse_position,
        } => {
            window_id.bus_right_button_down_entry(key_state, mouse_position);
            HandleResult::Handled
        }
        Message::RButtonUp {
            key_state,
            mouse_position,
        } => {
            window_id.bus_right_button_up_entry(key_state, mouse_position);
            HandleResult::Handled
        }
        Message::RButtonDoubleClick {
            key_state,
            mouse_position,
        } => {
            window_id.bus_right_button_double_click_entry(key_state, mouse_position);
            HandleResult::Handled
        }

        // ==================== 中键 (Middle Button) ====================
        Message::MButtonDown {
            key_state,
            mouse_position,
        } => {
            window_id.bus_middle_button_down_entry(key_state, mouse_position);
            HandleResult::Handled
        }
        Message::MButtonUp {
            key_state,
            mouse_position,
        } => {
            window_id.bus_middle_button_up_entry(key_state, mouse_position);
            HandleResult::Handled
        }
        Message::MButtonDoubleClick {
            key_state,
            mouse_position,
        } => {
            window_id.bus_middle_button_double_click_entry(key_state, mouse_position);
            HandleResult::Handled
        }
        Message::DpiChange { dpi_x, dpi_y } => {
            let Some(render_lock) = render(window_id) else {
                return Ok(HandleResult::Default);
            };
            render_lock.write().set_scale_factor(dpi_x, dpi_y)?;
            window_id.update_child_layout_dpi(dpi_x, dpi_y);
            HandleResult::Handled
        }
        Message::WindowDestroy => {
            trace!("event::WindowDestroy::remove_window");
            trace!("to_remove_window");
            remove_window(window_id);
            HandleResult::Handled
        }
        Message::MouseMove {
            key_state,
            mouse_position,
        } => {
            window_id.bus_mouse_move_entry(key_state, mouse_position);
            // window_id.request_redraw()?;
            HandleResult::Handled
        }
        Message::KeyDown {
            code,
            is_alt,
            is_ctrl,
            is_shift,
        } => {
            if code == KeyCode::Tab && !is_alt && !is_ctrl {
                if let Some(mut entry) = window_id.entry_mut() {
                    if is_shift {
                        trace!("focus_manager prev");
                        entry.focus_manager.prev()
                    } else {
                        trace!("focus_manager next");
                        entry.focus_manager.next()
                    }
                }
            }
            window_id.bus_key_down_entry(code, is_alt, is_ctrl, is_shift);
            HandleResult::Handled
        }
        Message::KeyUp {
            code,
            is_alt,
            is_ctrl,
            is_shift,
        } => {
            window_id.bus_key_up_entry(code, is_alt, is_ctrl, is_shift);
            HandleResult::Handled
        }
        Message::MouseLeave => {
            window_id.bus_mouse_leave_entry();
            HandleResult::Handled
        }
        Message::CloseRequested { prevent } => {
            *prevent = true;
            HandleResult::Handled
        }
        #[cfg(feature = "drag-drop")]
        Message::DragEnter {
            key_state,
            mouse_position,
            formats,
            effect,
        } => {
            *effect = window_id.bus_drag_enter_entry(key_state, mouse_position, formats);
            HandleResult::Handled
        }
        #[cfg(feature = "drag-drop")]
        Message::DragOver {
            key_state,
            mouse_position,
            formats,
            effect,
        } => {
            *effect = window_id.bus_drag_over_entry(key_state, mouse_position, formats);
            HandleResult::Handled
        }
        #[cfg(feature = "drag-drop")]
        Message::DragLeave => {
            window_id.bus_drag_leave_entry();
            HandleResult::Handled
        }
        #[cfg(feature = "drag-drop")]
        Message::Drop {
            key_state,
            mouse_position,
            data,
            effect,
        } => {
            *effect = window_id.bus_drop_entry(key_state, mouse_position, &data);
            HandleResult::Handled
        }
        Message::MouseWheel {
            axis,
            delta,
            key_state,
            mouse_position,
        } => {
            window_id.bus_wheel_scroll_lines_changed_entry(axis, delta, key_state, mouse_position);
            HandleResult::Handled
        }
        _ => HandleResult::Default,
    };
    Ok(handle_result)
}
