use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::entry::WindowEntryVisit;
use platform::WindowId;
use std::time::{Duration, Instant};

pub fn tooltip_check_entry(window_id: WindowId) -> Option<Duration> {
    let (should_show, remaining) = {
        let entry = window_id.entry()?;

        // 已经显示过了，不需要再检查
        if entry.tooltip_shown_for.is_some() {
            return None;
        }

        // 没有在计时
        let start = entry.tooltip_hover_start?;
        let elapsed = Instant::now().duration_since(start);

        if elapsed >= entry.tooltip_delay {
            (true, None)
        } else {
            (false, Some(entry.tooltip_delay - elapsed))
        }
    };

    if should_show {
        // 获取当前 hover 的控件并触发 tooltip_show
        let (hover_id, mouse_pos, key_state) = {
            let entry = window_id.entry()?;
            (
                entry.hover_id?,
                entry.last_mouse_position,
                entry.last_key_state,
            )
        };
        if let Some(view_lock) = VIEW_STORAGE.views.read().get(hover_id) {
            let local_pos = hover_id.window_to_local_position(mouse_pos);
            view_lock.write().call_tooltip_show(key_state, local_pos);
        }
        if let Some(mut entry) = window_id.entry_mut() {
            entry.tooltip_shown_for = Some(hover_id);
        }
        None
    } else {
        remaining
    }
}
