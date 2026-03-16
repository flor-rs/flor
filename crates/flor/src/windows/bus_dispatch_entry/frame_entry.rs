use crate::error::Error;
use crate::min_wait_time::MinWaitTime;
use crate::view::{ControlState, View, VIEW_STORAGE};
use platform::WindowId;
use std::time::{Duration, Instant};

pub fn frame_entry(window_id: WindowId) -> Result<Option<Duration>, Error> {
    let now = Instant::now();
    let root_id = window_id.view_id();

    let views = VIEW_STORAGE.views.read();
    let child_map = VIEW_STORAGE.child_ids.read();
    let states = VIEW_STORAGE.states.read();
    let visuals = VIEW_STORAGE.visual.read();

    let mut min_wait_time: Option<Duration> = None;
    let mut stack = vec![root_id];

    while let Some(view_id) = stack.pop() {
        // 跳过 display:none 的控件
        if let Some(state) = states.get(view_id) {
            let state = state.read();
            if state
                .layout_style
                .get_data_borrow(ControlState::Normal)
                .display
                == taffy::Display::None
            {
                continue;
            }
        }

        // 跳过不可见控件（上一帧未绘制）
        if visuals.get(view_id).is_none() && view_id != root_id {
            continue;
        }

        // 调用 on_frame
        if let Some(view) = views.get(view_id) {
            let child_wait_time = view.write().on_frame(now)?;
            min_wait_time.update_to_min_wait_time(child_wait_time);
        }

        // 将子节点压栈
        if let Some(children) = child_map.get(view_id) {
            for &child_id in children.iter() {
                stack.push(child_id);
            }
        }
    }

    Ok(min_wait_time)
}
