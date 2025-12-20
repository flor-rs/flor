use crate::view::view_id::ViewId;
use flor_platform_base::KeyCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct KeyHandler(pub Arc<dyn Fn(ViewId, KeyCode, bool, bool, bool) + Send + Sync + 'static>);

impl<F> From<F> for KeyHandler
where
    F: Fn(ViewId, KeyCode, bool, bool, bool) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        KeyHandler(Arc::new(f))
    }
}

pub type OnKeyDownHandler = KeyHandler;
pub type OnKeyUpHandler = KeyHandler;
