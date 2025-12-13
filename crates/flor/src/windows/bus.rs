use crate::error::Error;
use crate::log_error::LogError;
use crate::render::FlorRender;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::{WindowEntryVisit, WINDOW_ENTRY_MAP};
use crate::FlorGui;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use flor_graphics_base::RenderContext;
use flor_platform_base::{KeyCode, WindowOperations};
use log::{debug, trace};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use platform::base::HandleResult;
use platform::base::Message;
use platform::WindowId;
use std::ops::Deref;
use std::sync::Arc;

/// 总线存储结构
/// 存储窗口ID到(渲染器, ViewId)的映射
pub static RENDERS: Lazy<DashMap<WindowId, Arc<RwLock<FlorRender>>>> =
    Lazy::new(|| Default::default());

/// 注册窗口到总线
#[inline]
pub fn register_window(window_id: WindowId, render: FlorRender) {
    RENDERS.insert(window_id, Arc::new(RwLock::new(render)));
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
pub fn render<'a>(window_id: WindowId) -> Option<Ref<'a, WindowId, Arc<RwLock<FlorRender>>>> {
    RENDERS.get(&window_id)
}

pub fn render_from_view_id<'a>(
    view_id: ViewId,
) -> Option<Ref<'a, WindowId, Arc<RwLock<FlorRender>>>> {
    let ret = VIEW_STORAGE.window_ids.read().get(view_id).and_then(|window_id| RENDERS.get(&window_id));
    ret
}

/// 处理窗口事件
///
/// 小心访问bus卡死
///
pub fn event(mut window_id: WindowId, message: Message) -> Result<HandleResult, Error> {
    match message {
        Message::Draw => {
            trace!("Draw begin");
            if !window_id
                .entry()
                .map(|e| e.is_continuous_rendering())
                .unwrap_or(true)
            {
                window_id.bus_re_draw_entry()?;
            }
        }
        Message::Resize { width, height } => {
            window_id.bus_refresh_layout_entry()?;
            let bus = render(window_id);
            let Some(render) = bus.as_deref() else {
                return Ok(HandleResult::Default);
            };
            {
                let mut render = render.write();
                render
                    .update_window_size(width, height)
                    .log_error("fail update size");
            }
            if window_id
                .entry()
                .map(|e| e.is_continuous_rendering())
                .unwrap_or(true)
            {
                window_id.bus_re_draw_entry().log_error("fail draw");
            }
        }
        Message::WindowDestroy => {
            trace!("event::WindowDestroy::remove_window");
            trace!("to_remove_window");
            remove_window(window_id);
        }
        Message::MouseMove {
            key_state,
            mouse_position,
        } => {
            window_id.bus_mouse_move_entry(key_state, mouse_position);
            window_id.request_redraw()?;
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
        }
        Message::KeyUp {
            code,
            is_alt,
            is_ctrl,
            is_shift,
        } => {
            window_id.bus_key_up_entry(code, is_alt, is_ctrl, is_shift);
        }
        Message::MouseLeave => {
            window_id.bus_mouse_leave();
        }
        Message::Close => {
            return Ok(HandleResult::WindowClose(true));
        }
        _ => {
            // window_id.bus_event(message);
        }
    }

    Ok(HandleResult::Default)
}
