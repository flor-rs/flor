use crate::view::View;
use crate::view::VIEW_STORAGE;
use crate::windows::WindowBusDispatchEntry;
use flor_base::platform::{KeyState, MousePosition, ScrollAxis};
use log::{debug, warn};
use platform::WindowId;
use taffy::{Display, Overflow};

pub fn wheel_scroll_lines_changed_entry(
    window_id: WindowId,
    axis: ScrollAxis,
    delta: f32,
    key_state: KeyState,
    mouse_position: MousePosition,
) {
    let view_id = window_id.bus_hit_test_entry(mouse_position, key_state);
    debug!(
        "[Wheel] hit_test returned: {:?}, mouse_pos: ({}, {})",
        view_id, mouse_position.x, mouse_position.y
    );
    let mut parent_id = Some(view_id);

    let mut event_view_id = None;

    while let Some(view_id) = parent_id {
        match view_id.with_current_style(|style| {
            style.display != Display::None
                && (style.overflow.x == Overflow::Scroll || style.overflow.y == Overflow::Scroll)
        }) {
            Ok(is_overflow) => {
                if is_overflow {
                    event_view_id = Some(view_id);
                    break;
                }
            }
            Err(err) => {
                warn!("wheel_scroll_lines_changed_entry[ViewId: {{{}}}] Error calculating cursor style: {}",view_id , err);
            }
        }
        parent_id = view_id.parent_view_id();
    }
    let event_event_id = event_view_id.unwrap_or(window_id.view_id());
    if let Some(view) = VIEW_STORAGE.views.read().get(event_event_id) {
        view.write()
            .call_wheel_scroll_lines_changed(axis, delta, key_state, mouse_position);
    }
}
