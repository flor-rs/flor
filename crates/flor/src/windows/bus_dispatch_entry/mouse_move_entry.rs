use crate::view::VIEW_STORAGE;
use crate::windows::WindowBusDispatchEntry;
use crate::windows::WindowEntryVisit;
use flor_base::platform::{KeyState, MousePosition};
use log::trace;
use platform::WindowId;
use std::time::Instant;

pub fn mouse_move_entry(window_id: WindowId, key_state: KeyState, mouse_position: MousePosition) {
    // 获取捕获的控件（如果有）
    let capture_view_id = window_id.entry().and_then(|v| v.capture_view_id);

    // 1. 【获取新 ID】: 必定是一个有效的 ViewId (最差也是窗口自己)
    let new_hovered_id = window_id.bus_hit_test_entry(mouse_position, key_state);

    // 2. 【获取旧 ID】: 可能是 None (如果刚从窗口外移入)
    let old_hovered_id = window_id.entry().and_then(|v| v.hover_id);

    let views = VIEW_STORAGE.views.read();

    // =========================================================
    // 逻辑 A: 处理【离开】(MouseLeave)
    // 条件：之前有东西，且那个东西不是现在这个
    // 注意：即使有 capture，hover 状态仍然正常更新
    // =========================================================
    if let Some(old_id) = old_hovered_id {
        if old_id != new_hovered_id {
            // tooltip: 对旧控件隐藏 tooltip
            if window_id
                .entry()
                .map(|v| v.tooltip_shown_for == Some(old_id))
                .unwrap_or(false)
            {
                if let Some(view_lock) = views.get(old_id) {
                    view_lock.write().call_tooltip_hide();
                }
            }

            if let Some(view_lock) = views.get(old_id) {
                // 旧的离开（转换为控件局部坐标）
                let local_pos = old_id.window_to_local_position(mouse_position);
                view_lock.write().call_mouse_leave(key_state, local_pos);
            }
        }
    }

    // =========================================================
    // 逻辑 B: 处理【进入】(MouseEnter)
    // 条件：旧的是 None，或者 旧的 != 新的
    // 注意：即使有 capture，hover 状态仍然正常更新
    // =========================================================
    if old_hovered_id != Some(new_hovered_id) {
        if let Some(view_lock) = views.get(new_hovered_id) {
            // 新的进入（转换为控件局部坐标）
            let local_pos = new_hovered_id.window_to_local_position(mouse_position);
            view_lock.write().call_mouse_enter(key_state, local_pos);
        }
        // tooltip: 重置计时器
        if let Some(mut entry) = window_id.entry_mut() {
            entry.tooltip_hover_start = Some(Instant::now());
            entry.tooltip_shown_for = None;
        }
    }

    // =========================================================
    // 逻辑 C: 处理【移动】(MouseMove)
    // 如果有 capture，mouse_move 发给捕获的控件
    // 否则发给当前 hover 的控件
    // =========================================================
    let move_target_id = capture_view_id.unwrap_or(new_hovered_id);
    if let Some(view_lock) = views.get(move_target_id) {
        let local_pos = move_target_id.window_to_local_position(mouse_position);
        view_lock.write().call_mouse_move(key_state, local_pos);
    }

    // =========================================================
    // 3. 更新状态
    // =========================================================
    if let Some(mut entry) = window_id.entry_mut() {
        trace!("update hovered id {:?}", new_hovered_id);
        entry.hover_id = Some(new_hovered_id);
        entry.last_mouse_position = mouse_position;
        entry.last_key_state = key_state;
    }
    window_id.request_redraw();
}
