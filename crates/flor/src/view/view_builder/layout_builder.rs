use crate::log_error::ResultLogExt;
use crate::signal::effect::updater_effect::create_updater_with_id;
use crate::view::resolver::LayoutResolver;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;

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

        // 2. 创建响应式更新器
        let (effect_id, _) = create_updater_with_id(
            move || {
                let base_style = {
                    let states = VIEW_STORAGE.states.read();
                    let view_state = states
                        .get(view_id)
                        .expect(&format!("view[{}] not found ViewState", view_id))
                        .read();
                    view_state.layout_style.clone()
                };
                let current_base = base_style.clone().normal_layer();
                (style_fn)(current_base).normal_layer()
            },
            move |new_style| {
                {
                    let states = VIEW_STORAGE.states.read();
                    let mut view_state = states
                        .get(view_id)
                        .expect(&format!("view[{}] not found ViewState", view_id))
                        .write();

                    view_state.layout_style = new_style;
                }
                if let Some(window_id) = view_id.window_id() {
                    window_id.bus_re_draw_entry().error_on_err("fail draw");
                }
                view_id.request_redraw();
            },
        );
        view_id.pending_effect_id(effect_id);
        self
    }
}
