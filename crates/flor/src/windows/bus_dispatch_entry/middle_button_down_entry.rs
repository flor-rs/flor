use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::WindowEntryVisit;
use flor_base::platform::{KeyState, MousePosition};
use platform::WindowId;

pub fn middle_button_down_entry(
    window_id: WindowId,
    key_state: KeyState,
    mouse_position: MousePosition,
) {
    let view_id = window_id
        .entry()
        .map(|v| v.capture_view_id)
        .flatten()
        .unwrap_or(window_id.bus_hit_test_entry(mouse_position, key_state));
    if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
        window_id
            .entry_mut()
            .map(|mut v| v.m_down_view_id = Some(view_id));
        let local_pos = view_id.window_to_local_position(mouse_position);
        view.write().call_middle_button_down(key_state, local_pos);
        window_id.request_redraw();
    }
}
