use crate::log_error::ResultLogExt;
use crate::view::VIEW_STORAGE;
use crate::windows::WindowEntryVisit;
use platform::WindowId;

pub fn ime_start_entry(window_id: WindowId) {
    if let Some(view_id) = window_id
        .entry()
        .map(|v| v.focus_manager.current_view_id())
        .flatten()
    {
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .on_ime_start()
                .error_on_err(format!("on_ime_start {{ view_id:{} }}", view_id));
        }
    }
}
