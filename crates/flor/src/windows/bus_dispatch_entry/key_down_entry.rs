use crate::log_error::ResultLogExt;
use crate::view::{View, VIEW_STORAGE};
use crate::windows::WindowEntryVisit;
use flor_base::platform::{HandleResult, KeyCode};
use platform::WindowId;

pub fn key_down_entry(
    mut window_id: WindowId,
    code: KeyCode,
    is_alt: bool,
    is_ctrl: bool,
    is_shift: bool,
) -> HandleResult {
    let views = VIEW_STORAGE.views.read();

    if let Some(view_id) = window_id
        .entry()
        .and_then(|entry| entry.focus_manager.current_view_id())
    {
        if let Some(view) = views.get(view_id) {
            return view.write().call_key_down(code, is_alt, is_ctrl, is_shift);
        }
    }

    window_id
        .on_key_down(code, is_alt, is_ctrl, is_shift)
        .log_err(format!(
            "on_key_down {{ code: {:?}, is_alt: {:?}, is_ctrl: {:?}, is_shift: {:?} }}",
            code, is_alt, is_ctrl, is_shift
        ))
        .unwrap_or(HandleResult::Default)
}
