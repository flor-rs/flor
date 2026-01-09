use crate::view::handler::Handler;
use crate::view::view_id::ViewId;
use flor_platform_base::{KeyState, MousePosition, ScrollAxis};
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
