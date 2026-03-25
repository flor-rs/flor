use crate::view::VIEW_STORAGE;
use crate::windows::WindowBusDispatchEntry;
use crate::windows::WindowEntryVisit;
use flor_base::platform::{KeyState, MousePosition};
use platform::WindowId;

pub fn right_button_up_entry(
    window_id: WindowId,
    key_state: KeyState,
    mouse_position: MousePosition,
) {
    let (capture_view_id, r_down_view_id) = window_id
        .entry()
        .map(|v| (v.capture_view_id, v.r_down_view_id))
        .unwrap_or((None, None));

    let hit_view_id = window_id.bus_hit_test_entry(mouse_position, key_state);
    let target_view_id = capture_view_id.or(r_down_view_id).unwrap_or(hit_view_id);

    if let Some(view) = VIEW_STORAGE.views.read().get(target_view_id) {
        let local_pos = target_view_id.window_to_local_position(mouse_position);

        if r_down_view_id.is_some() && r_down_view_id == Some(hit_view_id) {
            view.write().call_right_button_click(key_state, local_pos);
        }

        view.write().call_right_button_up(key_state, local_pos);
        window_id.request_redraw();
    }
}
