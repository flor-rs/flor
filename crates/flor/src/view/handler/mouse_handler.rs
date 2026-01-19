use crate::view::view_id::ViewId;
use flor_base::platform::{KeyState, MousePosition};
use std::sync::Arc;

#[derive(Clone)]
pub struct MouseHandler(pub Arc<dyn Fn(ViewId, KeyState, MousePosition) + Send + Sync + 'static>);

impl<F> From<F> for MouseHandler
where
    F: Fn(ViewId, KeyState, MousePosition) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        MouseHandler(Arc::new(f))
    }
}

// mouse handler
pub type OnMouseMoveHandler = MouseHandler;
pub type OnDoubleClickHandler = MouseHandler;
pub type OnClickHandler = MouseHandler;
pub type OnButtonDownHandler = MouseHandler;
pub type OnButtonUpHandler = MouseHandler;
pub type OnRightButtonDoubleClickHandler = MouseHandler;
pub type OnRightButtonClickHandler = MouseHandler;
pub type OnRightButtonDownHandler = MouseHandler;
pub type OnRightButtonUpHandler = MouseHandler;
pub type OnMiddleButtonDoubleClickHandler = MouseHandler;
pub type OnMiddleButtonDownHandler = MouseHandler;
pub type OnMiddleButtonUpHandler = MouseHandler;
pub type OnContextMenuHandler = MouseHandler;