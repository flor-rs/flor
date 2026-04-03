use crate::log_error::ResultLogExt;
use crate::view::VIEW_STORAGE;
use crate::windows::WindowEntryVisit;
use flor_base::platform::InputEvent;
use platform::WindowId;

pub fn ime_input_entry(window_id: WindowId, input_event: InputEvent) {
    if let Some(view_id) = window_id
        .entry()
        .map(|v| v.focus_manager.current_view_id())
        .flatten()
    {
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .on_ime_input(input_event)
                .error_on_err(format!("on_ime_input"));
        }
    }
}
