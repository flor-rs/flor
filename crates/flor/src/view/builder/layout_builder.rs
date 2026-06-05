use crate::signal::create_updater_with_id;
use crate::view::resolver::LayoutResolver;
use crate::view::{View, VIEW_STORAGE};
use crate::windows::WindowEntryVisit;

pub trait LayoutBuilder {
    // 增加 'static 约束，因为闭包需要被 move 进 updater 长期持有
    fn layout<F>(self, style_fn: F) -> Self
    where
        F: Fn(LayoutResolver) -> LayoutResolver + 'static;
}

impl<T: View> LayoutBuilder for T {
    fn layout<F>(self, style_fn: F) -> Self
    where
        F: Fn(LayoutResolver) -> LayoutResolver + 'static,
    {
        let view_id = self.view_id();

        let layer_id = view_id.new_layout_resolver_layer();

        // 2. 创建响应式更新器
        let (effect_id, _) = create_updater_with_id(
            move || {
                let mut current_style = {
                    let states = VIEW_STORAGE.states.read();
                    let view_state = states
                        .get(view_id)
                        .expect(&format!("view[{}] not found ViewState", view_id))
                        .read();
                    view_state.layout_style.clone()
                };
                current_style.switch_layer(layer_id);
                (style_fn)(current_style)
            },
            move |mut new_style| {
                new_style.clear_cache();
                {
                    let states = VIEW_STORAGE.states.read();
                    let mut view_state = states
                        .get(view_id)
                        .expect(&format!("view[{}] not found ViewState", view_id))
                        .write();
                    {
                        //dbg!(new_style.cache_data.read().deref());
                    }
                    view_state.layout_style = new_style;
                }
                if let Some(window_id) = view_id.window_id() {
                    window_id.entry_mut().map(|entry| entry.mark_layout_dirty());
                }
                view_id.request_redraw();
            },
        );
        view_id.pending_effect_id(effect_id);
        self
    }
}
