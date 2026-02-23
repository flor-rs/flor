use crate::view::handler::Handler;
use crate::view::view_id::ViewId;
use flor_base::platform::{DragData, DragFormat, DropEffect, KeyState, MousePosition};
use std::sync::Arc;

#[derive(Clone)]
pub struct DragEnterOverHandler(
    pub  Arc<
        dyn Fn(ViewId, KeyState, MousePosition, &[DragFormat], &mut DropEffect)
            + Send
            + Sync
            + 'static,
    >,
);

impl<F> From<F> for DragEnterOverHandler
where
    F: Fn(ViewId, KeyState, MousePosition, &[DragFormat], &mut DropEffect) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        DragEnterOverHandler(Arc::new(f))
    }
}

pub type OnDragEnterHandler = DragEnterOverHandler;

pub type OnDragOverHandler = DragEnterOverHandler;

pub type OnDragLeaveHandler = Handler;

#[derive(Clone)]
pub struct DropHandler(
    pub  Arc<
        dyn Fn(ViewId, KeyState, MousePosition, &DragData, &mut DropEffect) + Send + Sync + 'static,
    >,
);

impl<F> From<F> for DropHandler
where
    F: Fn(ViewId, KeyState, MousePosition, &DragData, &mut DropEffect) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        DropHandler(Arc::new(f))
    }
}
