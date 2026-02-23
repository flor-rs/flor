use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::WindowEntryVisit;
use flor_base::platform::{KeyState, MousePosition};
use platform::WindowId;

pub fn right_button_up_entry(
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
        let local_pos = view_id.window_to_local_position(mouse_position);
        if let Some(spawn_click) = window_id.entry().map(|v| v.r_down_view_id == Some(view_id)) {
            if spawn_click {
                view.write().call_right_button_click(key_state, local_pos);
            }
        }
        view.write().call_right_button_up(key_state, local_pos);
        window_id.request_redraw();
    }
}
