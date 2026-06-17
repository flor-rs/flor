use crate::view::ViewId;
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

#[doc(hidden)]
pub struct FullArgs;

#[doc(hidden)]
pub struct NoArgs;

#[doc(hidden)]
pub struct ViewIdOnly;

#[doc(hidden)]
pub struct WithoutViewId;

pub trait IntoEventHandler<T, Args> {
    fn into_event_handler(self) -> T;
}

impl<T, F> IntoEventHandler<T, FullArgs> for F
where
    F: Into<T>,
{
    fn into_event_handler(self) -> T {
        self.into()
    }
}

#[derive(Default)]
pub struct ViewHandler {
    // mouse_handler
    pub on_mouse_move_handler: Option<OnMouseMoveHandler>,
    pub on_double_click_handler: Option<OnDoubleClickHandler>,
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
    pub on_create_handler: Option<OnCreateHandler>,
    pub on_destroy_handler: Option<OnDestroyHandler>,

    // tooltip_handler
    pub on_tooltip_show_handler: Option<OnTooltipShowHandler>,
    pub on_tooltip_hide_handler: Option<OnTooltipHideHandler>,

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
    pub on_drag_enter_handler: Option<OnDragEnterHandler>,
    #[cfg(feature = "drag-drop")]
    pub on_drag_over_handler: Option<OnDragOverHandler>,
    #[cfg(feature = "drag-drop")]
    pub on_drag_leave_handler: Option<OnDragLeaveHandler>,
    #[cfg(feature = "drag-drop")]
    pub on_drop_handler: Option<DropHandler>,
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
        field!(on_double_click_handler);
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
        field!(on_create_handler);
        field!(on_destroy_handler);

        field!(on_tooltip_show_handler);
        field!(on_tooltip_hide_handler);

        field!(on_resize_handler);
        field!(on_close_requested_handler);
        field!(on_work_area_changed_handler);
        field!(on_wheel_settings_changed_handler);
        field!(on_dpi_change_handler);

        #[cfg(feature = "theme-change")]
        field!(on_theme_changed_handler);

        #[cfg(feature = "drag-drop")]
        {
            field!(on_drag_enter_handler);
            field!(on_drag_over_handler);
            field!(on_drag_leave_handler);
            field!(on_drop_handler);
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

impl<F> IntoEventHandler<Handler, NoArgs> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_handler(self) -> Handler {
        Handler(Arc::new(move |_| self()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "theme-change")]
    use flor_base::platform::ThemeMode;
    use flor_base::platform::{HandleResult, KeyCode, KeyState, MousePosition, ScrollAxis};

    fn accept<T, Args>(handler: impl IntoEventHandler<T, Args>) -> T {
        handler.into_event_handler()
    }

    #[test]
    fn accepts_handler_parameter_shapes() {
        let _: Handler = accept(|_: ViewId| {});
        let _: Handler = accept(|| {});
    }

    #[test]
    fn accepts_key_handler_parameter_shapes() {
        let _: KeyHandler =
            accept(|_: ViewId, _: KeyCode, _: bool, _: bool, _: bool| HandleResult::Default);
        let _: KeyHandler = accept(|| HandleResult::Default);
        let _: KeyHandler = accept(|_: ViewId| HandleResult::Default);
        let _: KeyHandler = accept(|_: KeyCode, _: bool, _: bool, _: bool| HandleResult::Default);
    }

    #[test]
    fn accepts_mouse_handler_parameter_shapes() {
        let _: MouseHandler = accept(|_: ViewId, _: KeyState, _: MousePosition| {});
        let _: MouseHandler = accept(|| {});
        let _: MouseHandler = accept(|_: ViewId| {});
        let _: MouseHandler = accept(|_: KeyState, _: MousePosition| {});
    }

    #[test]
    fn accepts_focus_handler_parameter_shapes() {
        let _: FocusHandler = accept(|_: ViewId, _: u16| {});
        let _: FocusHandler = accept(|| {});
        let _: FocusHandler = accept(|_: ViewId| {});
        let _: FocusHandler = accept(|_: u16| {});
    }

    #[test]
    fn accepts_window_handler_parameter_shapes() {
        let _: OnWheelSettingsChangedHandler =
            accept(|_: ViewId, _: ScrollAxis, _: f32, _: KeyState, _: MousePosition| {});
        let _: OnWheelSettingsChangedHandler = accept(|| {});
        let _: OnWheelSettingsChangedHandler = accept(|_: ViewId| {});
        let _: OnWheelSettingsChangedHandler =
            accept(|_: ScrollAxis, _: f32, _: KeyState, _: MousePosition| {});

        let _: OnDpiChangeHandler = accept(|_: ViewId, _: f32, _: f32| {});
        let _: OnDpiChangeHandler = accept(|| {});
        let _: OnDpiChangeHandler = accept(|_: ViewId| {});
        let _: OnDpiChangeHandler = accept(|_: f32, _: f32| {});
    }

    #[cfg(feature = "theme-change")]
    #[test]
    fn accepts_theme_handler_parameter_shapes() {
        let _: OnThemeChangedHandler = accept(|_: ViewId, _: ThemeMode| {});
        let _: OnThemeChangedHandler = accept(|| {});
        let _: OnThemeChangedHandler = accept(|_: ViewId| {});
        let _: OnThemeChangedHandler = accept(|_: ThemeMode| {});
    }
}
