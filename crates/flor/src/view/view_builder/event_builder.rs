use crate::view::handler::{
    OnBlurHandler, OnButtonDownHandler, OnButtonUpHandler, OnClickHandler, OnCloseRequestedHandler,
    OnContextMenuHandler, OnCreateHandler, OnDestroyHandler, OnDoubleClickHandler,
    OnDpiChangeHandler, OnFocusHandler, OnKeyDownHandler, OnKeyUpHandler,
    OnMiddleButtonDoubleClickHandler, OnMiddleButtonDownHandler, OnMiddleButtonUpHandler,
    OnMouseEnterHandler, OnMouseLeaveHandler, OnMouseMoveHandler, OnResizeHandler,
    OnRightButtonClickHandler, OnRightButtonDoubleClickHandler, OnRightButtonDownHandler,
    OnRightButtonUpHandler, OnWheelSettingsChangedHandler, OnWorkAreaChangedHandler,
};

#[cfg(feature = "theme-change")]
use crate::view::handler::OnThemeChangedHandler;
#[cfg(feature = "drag-drop")]
use crate::view::handler::{
    DropHandler, OnDragEnterHandler, OnDragLeaveHandler, OnDragOverHandler,
};


use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;

pub trait EventBuilder {
    fn on_mouse_move(self, handler: impl Into<OnMouseMoveHandler>) -> Self;
    fn on_double_click(self, handler: impl Into<OnDoubleClickHandler>) -> Self;
    fn on_click(self, handler: impl Into<OnClickHandler>) -> Self;
    fn on_button_down(self, handler: impl Into<OnButtonDownHandler>) -> Self;
    fn on_button_up(self, handler: impl Into<OnButtonUpHandler>) -> Self;
    fn on_right_button_double_click(self, handler: impl Into<OnRightButtonDoubleClickHandler>) -> Self;
    fn on_right_button_click(self, handler: impl Into<OnRightButtonClickHandler>) -> Self;
    fn on_right_button_down(self, handler: impl Into<OnRightButtonDownHandler>) -> Self;
    fn on_right_button_up(self, handler: impl Into<OnRightButtonUpHandler>) -> Self;
    fn on_middle_button_double_click(self, handler: impl Into<OnMiddleButtonDoubleClickHandler>) -> Self;
    fn on_middle_button_down(self, handler: impl Into<OnMiddleButtonDownHandler>) -> Self;
    fn on_middle_button_up(self, handler: impl Into<OnMiddleButtonUpHandler>) -> Self;
    fn on_context_menu(self, handler: impl Into<OnContextMenuHandler>) -> Self;

    fn on_key_down(self, handler: impl Into<OnKeyDownHandler>) -> Self;
    fn on_key_up(self, handler: impl Into<OnKeyUpHandler>) -> Self;

    fn on_mouse_enter(self, handler: impl Into<OnMouseEnterHandler>) -> Self;
    fn on_mouse_leave(self, handler: impl Into<OnMouseLeaveHandler>) -> Self;
    fn on_focus(self, handler: impl Into<OnFocusHandler>) -> Self;
    fn on_blur(self, handler: impl Into<OnBlurHandler>) -> Self;
    fn on_create(self, handler: impl Into<OnCreateHandler>) -> Self;
    fn on_destroy(self, handler: impl Into<OnDestroyHandler>) -> Self;

    fn on_resize(self, handler: impl Into<OnResizeHandler>) -> Self;
    fn on_close_requested(self, handler: impl Into<OnCloseRequestedHandler>) -> Self;
    fn on_work_area_changed(self, handler: impl Into<OnWorkAreaChangedHandler>) -> Self;
    fn on_wheel_settings_changed(self, handler: impl Into<OnWheelSettingsChangedHandler>) -> Self;
    fn on_dpi_change(self, handler: impl Into<OnDpiChangeHandler>) -> Self;

    #[cfg(feature = "theme-change")]
    fn on_theme_changed(self, handler: impl Into<OnThemeChangedHandler>) -> Self;

    #[cfg(feature = "drag-drop")]
    fn on_drag_enter(self, handler: impl Into<OnDragEnterHandler>) -> Self;
    #[cfg(feature = "drag-drop")]
    fn on_drag_over(self, handler: impl Into<OnDragOverHandler>) -> Self;
    #[cfg(feature = "drag-drop")]
    fn on_drag_leave(self, handler: impl Into<OnDragLeaveHandler>) -> Self;
    #[cfg(feature = "drag-drop")]
    fn on_drop(self, handler: impl Into<DropHandler>) -> Self;
}

impl<V: View> EventBuilder for V {
    fn on_mouse_move(self, handler: impl Into<OnMouseMoveHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_mouse_move_handler = Some(handler.into());
        }
        self
    }

    fn on_double_click(self, handler: impl Into<OnDoubleClickHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_double_click_handler = Some(handler.into());
        }
        self
    }

    fn on_click(self, handler: impl Into<OnClickHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_click_handler = Some(handler.into());
        }
        self
    }

    fn on_button_down(self, handler: impl Into<OnButtonDownHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_button_down_handler = Some(handler.into());
        }
        self
    }

    fn on_button_up(self, handler: impl Into<OnButtonUpHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_button_up_handler = Some(handler.into());
        }
        self
    }

    fn on_right_button_double_click(
        self,
        handler: impl Into<OnRightButtonDoubleClickHandler>,
    ) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_right_button_double_click_handler = Some(handler.into());
        }
        self
    }

    fn on_right_button_click(self, handler: impl Into<OnRightButtonClickHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_right_button_click_handler = Some(handler.into());
        }
        self
    }

    fn on_right_button_down(self, handler: impl Into<OnRightButtonDownHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_right_button_down_handler = Some(handler.into());
        }
        self
    }

    fn on_right_button_up(self, handler: impl Into<OnRightButtonUpHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_right_button_up_handler = Some(handler.into());
        }
        self
    }

    fn on_middle_button_double_click(
        self,
        handler: impl Into<OnMiddleButtonDoubleClickHandler>,
    ) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_middle_button_double_click_handler = Some(handler.into());
        }
        self
    }

    fn on_middle_button_down(self, handler: impl Into<OnMiddleButtonDownHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_middle_button_down_handler = Some(handler.into());
        }
        self
    }

    fn on_middle_button_up(self, handler: impl Into<OnMiddleButtonUpHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_middle_button_up_handler = Some(handler.into());
        }
        self
    }

    fn on_context_menu(self, handler: impl Into<OnContextMenuHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_context_menu_handler = Some(handler.into());
        }
        self
    }

    fn on_key_down(self, handler: impl Into<OnKeyDownHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_key_down_handler = Some(handler.into());
        }
        self
    }

    fn on_key_up(self, handler: impl Into<OnKeyUpHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_key_up_handler = Some(handler.into());
        }
        self
    }

    fn on_mouse_enter(self, handler: impl Into<OnMouseEnterHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_mouse_enter_handler = Some(handler.into());
        }
        self
    }

    fn on_mouse_leave(self, handler: impl Into<OnMouseLeaveHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_mouse_leave_handler = Some(handler.into());
        }
        self
    }

    fn on_focus(self, handler: impl Into<OnFocusHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_focus_handler = Some(handler.into());
        }
        self
    }

    fn on_blur(self, handler: impl Into<OnBlurHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_blur_handler = Some(handler.into());
        }
        self
    }

    fn on_create(self, handler: impl Into<OnCreateHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_create_handler = Some(handler.into());
        }
        self
    }

    fn on_destroy(self, handler: impl Into<OnDestroyHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_destroy_handler = Some(handler.into());
        }
        self
    }

    fn on_resize(self, handler: impl Into<OnResizeHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_resize_handler = Some(handler.into());
        }
        self
    }

    fn on_close_requested(self, handler: impl Into<OnCloseRequestedHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_close_requested_handler = Some(handler.into());
        }
        self
    }

    fn on_work_area_changed(self, handler: impl Into<OnWorkAreaChangedHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_work_area_changed_handler = Some(handler.into());
        }
        self
    }

    fn on_wheel_settings_changed(self, handler: impl Into<OnWheelSettingsChangedHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_wheel_settings_changed_handler = Some(handler.into());
        }
        self
    }

    fn on_dpi_change(self, handler: impl Into<OnDpiChangeHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_dpi_change_handler = Some(handler.into());
        }
        self
    }

    #[cfg(feature = "theme-change")]
    fn on_theme_changed(self, handler: impl Into<OnThemeChangedHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_theme_changed_handler = Some(handler.into());
        }
        self
    }

    #[cfg(feature = "drag-drop")]
    fn on_drag_enter(self, handler: impl Into<OnDragEnterHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_drag_enter_handler = Some(handler.into());
        }
        self
    }

    #[cfg(feature = "drag-drop")]
    fn on_drag_over(self, handler: impl Into<OnDragOverHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_drag_over_handler = Some(handler.into());
        }
        self
    }

    #[cfg(feature = "drag-drop")]
    fn on_drag_leave(self, handler: impl Into<OnDragLeaveHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_drag_leave_handler = Some(handler.into());
        }
        self
    }

    #[cfg(feature = "drag-drop")]
    fn on_drop(self, handler: impl Into<DropHandler>) -> Self {
        let view_id = self.view_id();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_drop_handler = Some(handler.into());
        }
        self
    }
}
