use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::WindowEntryVisit;
use flor_base::platform::{KeyState, MousePosition};
use platform::WindowId;

pub fn button_down_entry(window_id: WindowId, key_state: KeyState, mouse_position: MousePosition) {
    // tooltip: 鼠标按下时隐藏 tooltip
    let tooltip_target = window_id.entry().and_then(|v| v.tooltip_shown_for);
    if let Some(view_id) = tooltip_target {
        if let Some(view_lock) = VIEW_STORAGE.views.read().get(view_id) {
            view_lock.write().call_tooltip_hide();
        }
        window_id.entry_mut().map(|mut v| {
            v.tooltip_hover_start = None;
            v.tooltip_shown_for = None;
        });
    }

    let view_id = window_id
        .entry()
        .map(|v| v.capture_view_id)
        .flatten()
        .unwrap_or(window_id.bus_hit_test_entry(mouse_position, key_state));
    if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
        window_id
            .entry_mut()
            .map(|mut v| v.l_down_view_id = Some(view_id));
        VIEW_STORAGE.pressed.write().insert(view_id, ());
        // 转换为控件局部坐标
        let local_pos = view_id.window_to_local_position(mouse_position);
        view.write().call_button_down(key_state, local_pos);
        window_id.request_redraw();
    }
}
