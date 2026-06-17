use crate::view::handler::{Handler, IntoEventHandler, NoArgs, ViewIdOnly, WithoutViewId};
use crate::view::ViewId;
use flor_base::platform::{KeyState, MousePosition, ScrollAxis};
use std::sync::Arc;

pub type OnResizeHandler = Handler;
pub type OnCloseRequestedHandler = Handler;
pub type OnWorkAreaChangedHandler = Handler;

#[derive(Clone)]
pub struct OnWheelSettingsChangedHandler(
    pub Arc<dyn Fn(ViewId, ScrollAxis, f32, KeyState, MousePosition) + Send + Sync + 'static>,
);

impl<F> From<F> for OnWheelSettingsChangedHandler
where
    F: Fn(ViewId, ScrollAxis, f32, KeyState, MousePosition) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        OnWheelSettingsChangedHandler(Arc::new(f))
    }
}

impl<F> IntoEventHandler<OnWheelSettingsChangedHandler, NoArgs> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnWheelSettingsChangedHandler {
        OnWheelSettingsChangedHandler(Arc::new(move |_, _, _, _, _| self()))
    }
}

impl<F> IntoEventHandler<OnWheelSettingsChangedHandler, ViewIdOnly> for F
where
    F: Fn(ViewId) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnWheelSettingsChangedHandler {
        OnWheelSettingsChangedHandler(Arc::new(move |view_id, _, _, _, _| self(view_id)))
    }
}

impl<F> IntoEventHandler<OnWheelSettingsChangedHandler, WithoutViewId> for F
where
    F: Fn(ScrollAxis, f32, KeyState, MousePosition) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnWheelSettingsChangedHandler {
        OnWheelSettingsChangedHandler(Arc::new(
            move |_, axis, delta, key_state, mouse_position| {
                self(axis, delta, key_state, mouse_position)
            },
        ))
    }
}

#[derive(Clone)]
pub struct OnDpiChangeHandler(pub Arc<dyn Fn(ViewId, f32, f32) + Send + Sync + 'static>);

impl<F> From<F> for OnDpiChangeHandler
where
    F: Fn(ViewId, f32, f32) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        OnDpiChangeHandler(Arc::new(f))
    }
}

impl<F> IntoEventHandler<OnDpiChangeHandler, NoArgs> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnDpiChangeHandler {
        OnDpiChangeHandler(Arc::new(move |_, _, _| self()))
    }
}

impl<F> IntoEventHandler<OnDpiChangeHandler, ViewIdOnly> for F
where
    F: Fn(ViewId) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnDpiChangeHandler {
        OnDpiChangeHandler(Arc::new(move |view_id, _, _| self(view_id)))
    }
}

impl<F> IntoEventHandler<OnDpiChangeHandler, WithoutViewId> for F
where
    F: Fn(f32, f32) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnDpiChangeHandler {
        OnDpiChangeHandler(Arc::new(move |_, dpi_x, dpi_y| self(dpi_x, dpi_y)))
    }
}
