use crate::view::handler::{Handler, IntoEventHandler, NoArgs, ViewIdOnly, WithoutViewId};
use crate::view::ViewId;
use std::sync::Arc;

#[derive(Clone)]
pub struct FocusHandler(pub Arc<dyn Fn(ViewId, u16) + Send + Sync + 'static>);

impl<F> From<F> for FocusHandler
where
    F: Fn(ViewId, u16) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        FocusHandler(Arc::new(f))
    }
}

impl<F> IntoEventHandler<FocusHandler, NoArgs> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_handler(self) -> FocusHandler {
        FocusHandler(Arc::new(move |_, _| self()))
    }
}

impl<F> IntoEventHandler<FocusHandler, ViewIdOnly> for F
where
    F: Fn(ViewId) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> FocusHandler {
        FocusHandler(Arc::new(move |view_id, _| self(view_id)))
    }
}

impl<F> IntoEventHandler<FocusHandler, WithoutViewId> for F
where
    F: Fn(u16) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> FocusHandler {
        FocusHandler(Arc::new(move |_, focus_index| self(focus_index)))
    }
}

pub type OnMouseEnterHandler = Handler;
pub type OnMouseLeaveHandler = Handler;
pub type OnFocusHandler = FocusHandler;
pub type OnBlurHandler = FocusHandler;
pub type OnCreateHandler = Handler;
pub type OnDestroyHandler = Handler;

// tooltip_handler
pub type OnTooltipShowHandler = super::MouseHandler;
pub type OnTooltipHideHandler = Handler;
