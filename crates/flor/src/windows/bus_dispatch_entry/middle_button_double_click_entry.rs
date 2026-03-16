use crate::view::VIEW_STORAGE;
use crate::windows::WindowBusDispatchEntry;
use crate::windows::WindowEntryVisit;
use flor_base::platform::{KeyState, MousePosition};
use platform::WindowId;

pub fn middle_button_double_click_entry(
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
        // DBLCLK 在 Windows 上替代了 MBUTTONDOWN，所以也需要设置 m_down_view_id
        window_id
            .entry_mut()
            .map(|mut v| v.m_down_view_id = Some(view_id));
        let local_pos = view_id.window_to_local_position(mouse_position);
        view.write()
            .call_middle_button_double_click(key_state, local_pos);
        window_id.request_redraw();
    }
}
