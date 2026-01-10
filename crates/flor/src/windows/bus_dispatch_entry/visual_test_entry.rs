use crate::view::view_storage::VIEW_STORAGE;
use platform::WindowId;

pub fn visual_test_entry(window_id: WindowId) {
    let z_index_sort_guard = VIEW_STORAGE.z_index_sort.read();
    let mut visual = VIEW_STORAGE.visual.write();
    let view_guard = VIEW_STORAGE.views.read();
    if let Some(view_ids) = z_index_sort_guard.get(&window_id) {
        for view_id in view_ids {
            if let Some(view) = view_guard.get(*view_id) {
                let view = view.read();
                let visual_rect = view.visual_rect();
                let view_visual = view.on_visual_test_entry(visual_rect);
                visual.insert(*view_id, view_visual);
            }
        }
    }
}
