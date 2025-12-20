use crate::view::view_id::ViewId;
use std::sync::Arc;

#[cfg(feature = "drag-drop")]
mod drag_drop_handler;
mod key_handler;
mod mouse_handler;
mod view_handler;
mod window_handler;
#[cfg(feature = "drag-drop")]
pub use drag_drop_handler::*;
#[cfg(feature = "theme-change")]
mod theme_change_handler;
#[cfg(feature = "theme-change")]
pub use theme_change_handler::*;

pub use {key_handler::*, mouse_handler::*, view_handler::*, window_handler::*};

#[derive(Default)]
pub struct ViewHandler {
    // mouse_handler
    pub on_mouse_move_handler: Option<OnMouseMoveHandler>,
    pub on_l_button_double_click_handler: Option<OnLButtonDoubleClickHandler>,
    pub on_click_handler: Option<OnClickHandler>,
    pub on_button_down_handler: Option<OnButtonDownHandler>,
    pub on_button_up_handler: Option<OnButtonUpHandler>,
    pub on_right_button_double_click_handler: Option<OnRightButtonDoubleClickHandler>,
    pub on_right_button_click_handler: Option<OnRightButtonClickHandler>,
    pub on_right_button_down_handler: Option<OnRightButtonDownHandler>,
    pub on_right_button_up_handler: Option<OnRightButtonUpHandler>,
    pub on_middle_button_double_click_handler: Option<OnMiddleButtonDoubleClickHandler>,
    pub on_middle_button_down_handler: Option<OnMiddleButtonDownHandler>,
    pub on_middle_button_up_handler: Option<OnMiddleButtonUpHandler>,
    pub on_context_menu_handler: Option<OnContextMenuHandler>,

    // key_handler
    pub on_key_down_handler: Option<OnKeyDownHandler>,
    pub on_key_up_handler: Option<OnKeyUpHandler>,

    // view_handler
    pub on_mouse_enter_handler: Option<OnMouseEnterHandler>,
    pub on_mouse_leave_handler: Option<OnMouseLeaveHandler>,
    pub on_focus_handler: Option<OnFocusHandler>,
    pub on_blur_handler: Option<OnBlurHandler>,
    pub on_drag_start_handler: Option<OnDragStartHandler>,
    pub on_drag_enter_handler: Option<OnDragEnterHandler>,
    pub on_drag_leave_handler: Option<OnDragLeaveHandler>,
    pub on_drop_handler: Option<OnDropHandler>,
    pub on_create_handler: Option<OnCreateHandler>,
    pub on_destroy_handler: Option<OnDestroyHandler>,

    // window_handler
    pub on_resize_handler: Option<OnResizeHandler>,
    pub on_close_requested_handler: Option<OnCloseRequestedHandler>,
    pub on_work_area_changed_handler: Option<OnWorkAreaChangedHandler>,
    pub on_wheel_settings_changed_handler: Option<OnWheelSettingsChangedHandler>,
    pub on_dpi_change_handler: Option<OnDpiChangeHandler>,

    // theme_change_handler
    #[cfg(feature = "theme-change")]
    pub on_theme_changed_handler: Option<OnThemeChangedHandler>,

    // drag_drop_handler
    #[cfg(feature = "drag-drop")]
    pub drag_enter_handler: Option<DragEnterHandler>,
    #[cfg(feature = "drag-drop")]
    pub drag_over_handler: Option<DragOverHandler>,
    #[cfg(feature = "drag-drop")]
    pub on_drag_leave: Option<OnDragLeave>,
    #[cfg(feature = "drag-drop")]
    pub drop_handler: Option<DropHandler>,
}

impl std::fmt::Debug for ViewHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("ViewHandler");

        macro_rules! field {
            ($name:ident) => {
                debug.field(stringify!($name), &self.$name.is_some());
            };
        }

        field!(on_mouse_move_handler);
        field!(on_l_button_double_click_handler);
        field!(on_click_handler);
        field!(on_button_down_handler);
        field!(on_button_up_handler);
        field!(on_right_button_double_click_handler);
        field!(on_right_button_click_handler);
        field!(on_right_button_down_handler);
        field!(on_right_button_up_handler);
        field!(on_middle_button_double_click_handler);
        field!(on_middle_button_down_handler);
        field!(on_middle_button_up_handler);
        field!(on_context_menu_handler);

        field!(on_key_down_handler);
        field!(on_key_up_handler);

        field!(on_mouse_enter_handler);
        field!(on_mouse_leave_handler);
        field!(on_focus_handler);
        field!(on_blur_handler);
        field!(on_drag_start_handler);
        field!(on_drag_enter_handler);
        field!(on_drag_leave_handler);
        field!(on_drop_handler);
        field!(on_create_handler);
        field!(on_destroy_handler);

        field!(on_resize_handler);
        field!(on_close_requested_handler);
        field!(on_work_area_changed_handler);
        field!(on_wheel_settings_changed_handler);
        field!(on_dpi_change_handler);

        #[cfg(feature = "theme-change")]
        field!(on_theme_changed_handler);

        #[cfg(feature = "drag-drop")]
        {
            field!(drag_enter_handler);
            field!(drag_over_handler);
            field!(on_drag_leave);
            field!(drop_handler);
        }

        debug.finish()
    }
}

#[derive(Clone)]
pub struct Handler(pub Arc<dyn Fn(ViewId) + Send + Sync + 'static>);

impl<F> From<F> for Handler
where
    F: Fn(ViewId) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        Handler(Arc::new(f))
    }
}
