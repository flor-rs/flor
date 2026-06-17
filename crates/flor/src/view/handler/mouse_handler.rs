use crate::view::handler::{IntoEventHandler, NoArgs, ViewIdOnly, WithoutViewId};
use crate::view::ViewId;
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

impl<F> IntoEventHandler<MouseHandler, NoArgs> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_handler(self) -> MouseHandler {
        MouseHandler(Arc::new(move |_, _, _| self()))
    }
}

impl<F> IntoEventHandler<MouseHandler, ViewIdOnly> for F
where
    F: Fn(ViewId) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> MouseHandler {
        MouseHandler(Arc::new(move |view_id, _, _| self(view_id)))
    }
}

impl<F> IntoEventHandler<MouseHandler, WithoutViewId> for F
where
    F: Fn(KeyState, MousePosition) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> MouseHandler {
        MouseHandler(Arc::new(move |_, key_state, mouse_position| {
            self(key_state, mouse_position)
        }))
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
