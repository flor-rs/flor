use crate::view::handler::{
    IntoEventHandler, OnBlurHandler, OnButtonDownHandler, OnButtonUpHandler, OnClickHandler,
    OnCloseRequestedHandler, OnContextMenuHandler, OnCreateHandler, OnDestroyHandler,
    OnDoubleClickHandler, OnDpiChangeHandler, OnFocusHandler, OnKeyDownHandler, OnKeyUpHandler,
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

use crate::view::{ViewIdentity, VIEW_STORAGE};

pub trait EventBuilder {
    fn on_mouse_move<Args>(self, handler: impl IntoEventHandler<OnMouseMoveHandler, Args>) -> Self;
    fn on_double_click<Args>(
        self,
        handler: impl IntoEventHandler<OnDoubleClickHandler, Args>,
    ) -> Self;
    fn on_click<Args>(self, handler: impl IntoEventHandler<OnClickHandler, Args>) -> Self;
    fn on_button_down<Args>(
        self,
        handler: impl IntoEventHandler<OnButtonDownHandler, Args>,
    ) -> Self;
    fn on_button_up<Args>(self, handler: impl IntoEventHandler<OnButtonUpHandler, Args>) -> Self;
    fn on_right_button_double_click<Args>(
        self,
        handler: impl IntoEventHandler<OnRightButtonDoubleClickHandler, Args>,
    ) -> Self;
    fn on_right_button_click<Args>(
        self,
        handler: impl IntoEventHandler<OnRightButtonClickHandler, Args>,
    ) -> Self;
    fn on_right_button_down<Args>(
        self,
        handler: impl IntoEventHandler<OnRightButtonDownHandler, Args>,
    ) -> Self;
    fn on_right_button_up<Args>(
        self,
        handler: impl IntoEventHandler<OnRightButtonUpHandler, Args>,
    ) -> Self;
    fn on_middle_button_double_click<Args>(
        self,
        handler: impl IntoEventHandler<OnMiddleButtonDoubleClickHandler, Args>,
    ) -> Self;
    fn on_middle_button_down<Args>(
        self,
        handler: impl IntoEventHandler<OnMiddleButtonDownHandler, Args>,
    ) -> Self;
    fn on_middle_button_up<Args>(
        self,
        handler: impl IntoEventHandler<OnMiddleButtonUpHandler, Args>,
    ) -> Self;
    fn on_context_menu<Args>(
        self,
        handler: impl IntoEventHandler<OnContextMenuHandler, Args>,
    ) -> Self;

    fn on_key_down<Args>(self, handler: impl IntoEventHandler<OnKeyDownHandler, Args>) -> Self;
    fn on_key_up<Args>(self, handler: impl IntoEventHandler<OnKeyUpHandler, Args>) -> Self;

    fn on_mouse_enter<Args>(
        self,
        handler: impl IntoEventHandler<OnMouseEnterHandler, Args>,
    ) -> Self;
    fn on_mouse_leave<Args>(
        self,
        handler: impl IntoEventHandler<OnMouseLeaveHandler, Args>,
    ) -> Self;
    fn on_focus<Args>(self, handler: impl IntoEventHandler<OnFocusHandler, Args>) -> Self;
    fn on_blur<Args>(self, handler: impl IntoEventHandler<OnBlurHandler, Args>) -> Self;
    fn on_create<Args>(self, handler: impl IntoEventHandler<OnCreateHandler, Args>) -> Self;
    fn on_destroy<Args>(self, handler: impl IntoEventHandler<OnDestroyHandler, Args>) -> Self;

    fn on_resize<Args>(self, handler: impl IntoEventHandler<OnResizeHandler, Args>) -> Self;
    fn on_close_requested<Args>(
        self,
        handler: impl IntoEventHandler<OnCloseRequestedHandler, Args>,
    ) -> Self;
    fn on_work_area_changed<Args>(
        self,
        handler: impl IntoEventHandler<OnWorkAreaChangedHandler, Args>,
    ) -> Self;
    fn on_wheel_settings_changed<Args>(
        self,
        handler: impl IntoEventHandler<OnWheelSettingsChangedHandler, Args>,
    ) -> Self;
    fn on_dpi_change<Args>(self, handler: impl IntoEventHandler<OnDpiChangeHandler, Args>) -> Self;

    #[cfg(feature = "theme-change")]
    fn on_theme_changed<Args>(
        self,
        handler: impl IntoEventHandler<OnThemeChangedHandler, Args>,
    ) -> Self;

    #[cfg(feature = "drag-drop")]
    fn on_drag_enter<Args>(self, handler: impl IntoEventHandler<OnDragEnterHandler, Args>) -> Self;
    #[cfg(feature = "drag-drop")]
    fn on_drag_over<Args>(self, handler: impl IntoEventHandler<OnDragOverHandler, Args>) -> Self;
    #[cfg(feature = "drag-drop")]
    fn on_drag_leave<Args>(self, handler: impl IntoEventHandler<OnDragLeaveHandler, Args>) -> Self;
    #[cfg(feature = "drag-drop")]
    fn on_drop<Args>(self, handler: impl IntoEventHandler<DropHandler, Args>) -> Self;
}

impl<V: ViewIdentity> EventBuilder for V {
    fn on_mouse_move<Args>(self, handler: impl IntoEventHandler<OnMouseMoveHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_mouse_move_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_double_click<Args>(
        self,
        handler: impl IntoEventHandler<OnDoubleClickHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_double_click_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_click<Args>(self, handler: impl IntoEventHandler<OnClickHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_click_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_button_down<Args>(
        self,
        handler: impl IntoEventHandler<OnButtonDownHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_button_down_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_button_up<Args>(self, handler: impl IntoEventHandler<OnButtonUpHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_button_up_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_right_button_double_click<Args>(
        self,
        handler: impl IntoEventHandler<OnRightButtonDoubleClickHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_right_button_double_click_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_right_button_click<Args>(
        self,
        handler: impl IntoEventHandler<OnRightButtonClickHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_right_button_click_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_right_button_down<Args>(
        self,
        handler: impl IntoEventHandler<OnRightButtonDownHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_right_button_down_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_right_button_up<Args>(
        self,
        handler: impl IntoEventHandler<OnRightButtonUpHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_right_button_up_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_middle_button_double_click<Args>(
        self,
        handler: impl IntoEventHandler<OnMiddleButtonDoubleClickHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_middle_button_double_click_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_middle_button_down<Args>(
        self,
        handler: impl IntoEventHandler<OnMiddleButtonDownHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_middle_button_down_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_middle_button_up<Args>(
        self,
        handler: impl IntoEventHandler<OnMiddleButtonUpHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_middle_button_up_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_context_menu<Args>(
        self,
        handler: impl IntoEventHandler<OnContextMenuHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_context_menu_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_key_down<Args>(self, handler: impl IntoEventHandler<OnKeyDownHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_key_down_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_key_up<Args>(self, handler: impl IntoEventHandler<OnKeyUpHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_key_up_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_mouse_enter<Args>(
        self,
        handler: impl IntoEventHandler<OnMouseEnterHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_mouse_enter_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_mouse_leave<Args>(
        self,
        handler: impl IntoEventHandler<OnMouseLeaveHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_mouse_leave_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_focus<Args>(self, handler: impl IntoEventHandler<OnFocusHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_focus_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_blur<Args>(self, handler: impl IntoEventHandler<OnBlurHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_blur_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_create<Args>(self, handler: impl IntoEventHandler<OnCreateHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_create_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_destroy<Args>(self, handler: impl IntoEventHandler<OnDestroyHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_destroy_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_resize<Args>(self, handler: impl IntoEventHandler<OnResizeHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_resize_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_close_requested<Args>(
        self,
        handler: impl IntoEventHandler<OnCloseRequestedHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_close_requested_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_work_area_changed<Args>(
        self,
        handler: impl IntoEventHandler<OnWorkAreaChangedHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_work_area_changed_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_wheel_settings_changed<Args>(
        self,
        handler: impl IntoEventHandler<OnWheelSettingsChangedHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_wheel_settings_changed_handler = Some(handler.into_event_handler());
        }
        self
    }

    fn on_dpi_change<Args>(self, handler: impl IntoEventHandler<OnDpiChangeHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_dpi_change_handler = Some(handler.into_event_handler());
        }
        self
    }

    #[cfg(feature = "theme-change")]
    fn on_theme_changed<Args>(
        self,
        handler: impl IntoEventHandler<OnThemeChangedHandler, Args>,
    ) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_theme_changed_handler = Some(handler.into_event_handler());
        }
        self
    }

    #[cfg(feature = "drag-drop")]
    fn on_drag_enter<Args>(self, handler: impl IntoEventHandler<OnDragEnterHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_drag_enter_handler = Some(handler.into_event_handler());
        }
        self
    }

    #[cfg(feature = "drag-drop")]
    fn on_drag_over<Args>(self, handler: impl IntoEventHandler<OnDragOverHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_drag_over_handler = Some(handler.into_event_handler());
        }
        self
    }

    #[cfg(feature = "drag-drop")]
    fn on_drag_leave<Args>(self, handler: impl IntoEventHandler<OnDragLeaveHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_drag_leave_handler = Some(handler.into_event_handler());
        }
        self
    }

    #[cfg(feature = "drag-drop")]
    fn on_drop<Args>(self, handler: impl IntoEventHandler<DropHandler, Args>) -> Self {
        let view_id = self.identity();
        let states = VIEW_STORAGE.handlers.read();
        if let Some(view_state) = states.get(view_id) {
            let mut vs = view_state.write();
            vs.on_drop_handler = Some(handler.into_event_handler());
        }
        self
    }
}
