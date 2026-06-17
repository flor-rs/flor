use crate::view::handler::{IntoEventHandler, NoArgs, ViewIdOnly, WithoutViewId};
use crate::view::ViewId;
use flor_base::platform::{HandleResult, KeyCode};
use std::sync::Arc;

#[derive(Clone)]
pub struct KeyHandler(
    pub Arc<dyn Fn(ViewId, KeyCode, bool, bool, bool) -> HandleResult + Send + Sync + 'static>,
);

impl<F> From<F> for KeyHandler
where
    F: Fn(ViewId, KeyCode, bool, bool, bool) -> HandleResult + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        KeyHandler(Arc::new(f))
    }
}

impl<F> IntoEventHandler<KeyHandler, NoArgs> for F
where
    F: Fn() -> HandleResult + Send + Sync + 'static,
{
    fn into_event_handler(self) -> KeyHandler {
        KeyHandler(Arc::new(move |_, _, _, _, _| self()))
    }
}

impl<F> IntoEventHandler<KeyHandler, ViewIdOnly> for F
where
    F: Fn(ViewId) -> HandleResult + Send + Sync + 'static,
{
    fn into_event_handler(self) -> KeyHandler {
        KeyHandler(Arc::new(move |view_id, _, _, _, _| self(view_id)))
    }
}

impl<F> IntoEventHandler<KeyHandler, WithoutViewId> for F
where
    F: Fn(KeyCode, bool, bool, bool) -> HandleResult + Send + Sync + 'static,
{
    fn into_event_handler(self) -> KeyHandler {
        KeyHandler(Arc::new(move |_, code, is_alt, is_ctrl, is_shift| {
            self(code, is_alt, is_ctrl, is_shift)
        }))
    }
}

pub type OnKeyDownHandler = KeyHandler;
pub type OnKeyUpHandler = KeyHandler;
