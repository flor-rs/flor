use crate::view::handler::Handler;
use crate::view::view_id::ViewId;
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

pub type OnMouseEnterHandler = Handler;
pub type OnMouseLeaveHandler = Handler;
pub type OnFocusHandler = FocusHandler;
pub type OnBlurHandler = FocusHandler;
pub type OnCreateHandler = Handler;
pub type OnDestroyHandler = Handler;

// tooltip_handler
pub type OnTooltipShowHandler = super::MouseHandler;
pub type OnTooltipHideHandler = Handler;
