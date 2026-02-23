use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::WindowEntryVisit;
use platform::WindowId;

pub fn mouse_leave_entry(window_id: WindowId) {
    // tooltip: 鼠标离开窗口，隐藏 tooltip
    let tooltip_target = window_id.entry().and_then(|v| v.tooltip_shown_for);
    if let Some(view_id) = tooltip_target {
        if let Some(view_lock) = VIEW_STORAGE.views.read().get(view_id) {
            view_lock.write().call_tooltip_hide();
        }
    }

    window_id.entry_mut().map(|mut v| {
        v.tooltip_hover_start = None;
        v.tooltip_shown_for = None;
        if v.hover_id != None {
            v.hover_id = None;
            window_id.request_redraw();
        }
    });
}
