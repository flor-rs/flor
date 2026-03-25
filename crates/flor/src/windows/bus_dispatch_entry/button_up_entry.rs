use crate::view::VIEW_STORAGE;
use crate::windows::{WindowBusDispatchEntry, WindowEntryVisit};
use flor_base::platform::{KeyState, MousePosition};
use platform::WindowId;

pub fn button_up_entry(window_id: WindowId, key_state: KeyState, mouse_position: MousePosition) {
    let (capture_view_id, l_down_view_id) = window_id
        .entry()
        .map(|v| (v.capture_view_id, v.l_down_view_id))
        .unwrap_or((None, None));

    // 当前命中结果用于判定是否需要派发 Click（合成事件）
    let hit_view_id = window_id.bus_hit_test_entry(mouse_position, key_state);

    // 原始 Up 事件需要投递给按下的控件（或 capture 的控件），避免漏发
    let target_view_id = capture_view_id.or(l_down_view_id).unwrap_or(hit_view_id);

    // 按下控件才有 pressed 状态，松开时必须移除
    if let Some(down_id) = l_down_view_id {
        VIEW_STORAGE.pressed.write().remove(down_id);
    }

    if let Some(view) = VIEW_STORAGE.views.read().get(target_view_id) {
        let local_pos = target_view_id.window_to_local_position(mouse_position);

        // 合成 Click 仅在“按下 + 松开”都落在同一命中控件时触发
        if l_down_view_id.is_some() && l_down_view_id == Some(hit_view_id) {
            let mut view = view.write();
            view.call_click(key_state, local_pos);
            let virtual_focus = view.on_virtual_focus_at(key_state, local_pos);
            drop(view);
            target_view_id.set_focus(Some(virtual_focus));
        }

        view.write().call_button_up(key_state, local_pos);
        window_id.request_redraw();
    }
}
