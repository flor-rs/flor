use crate::view::VIEW_STORAGE;
use crate::windows::{WindowBusDispatchEntry, WindowEntryVisit};
use flor_base::platform::{KeyState, MousePosition};
use platform::WindowId;

pub fn button_up_entry(window_id: WindowId, key_state: KeyState, mouse_position: MousePosition) {
    let view_id = window_id
        .entry()
        .map(|v| v.capture_view_id)
        .flatten()
        .unwrap_or(window_id.bus_hit_test_entry(mouse_position, key_state));
    if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
        // 转换为控件局部坐标
        let local_pos = view_id.window_to_local_position(mouse_position);
        // 合成事件，点击
        if let Some(spawn_click) = window_id.entry().map(|v| v.l_down_view_id == Some(view_id)) {
            if spawn_click {
                let mut view = view.write();
                view.call_click(key_state, local_pos);
                let virtual_focus = view.on_virtual_focus_at(key_state, local_pos);
                drop(view);
                view_id.set_focus(Some(virtual_focus));
            }
        }
        VIEW_STORAGE.pressed.write().remove(view_id);
        view.write().call_button_up(key_state, local_pos);
        window_id.request_redraw();
    }
}
