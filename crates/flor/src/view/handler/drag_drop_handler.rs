use crate::view::handler::{Handler, IntoEventHandler, NoArgs, ViewIdOnly, WithoutViewId};
use crate::view::ViewId;
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

impl<F> IntoEventHandler<DragEnterOverHandler, NoArgs> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_handler(self) -> DragEnterOverHandler {
        DragEnterOverHandler(Arc::new(move |_, _, _, _, _| self()))
    }
}

impl<F> IntoEventHandler<DragEnterOverHandler, ViewIdOnly> for F
where
    F: Fn(ViewId) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> DragEnterOverHandler {
        DragEnterOverHandler(Arc::new(move |view_id, _, _, _, _| self(view_id)))
    }
}

impl<F> IntoEventHandler<DragEnterOverHandler, WithoutViewId> for F
where
    F: Fn(KeyState, MousePosition, &[DragFormat], &mut DropEffect) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> DragEnterOverHandler {
        DragEnterOverHandler(Arc::new(
            move |_, key_state, mouse_position, formats, effect| {
                self(key_state, mouse_position, formats, effect)
            },
        ))
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

impl<F> IntoEventHandler<DropHandler, NoArgs> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_handler(self) -> DropHandler {
        DropHandler(Arc::new(move |_, _, _, _, _| self()))
    }
}

impl<F> IntoEventHandler<DropHandler, ViewIdOnly> for F
where
    F: Fn(ViewId) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> DropHandler {
        DropHandler(Arc::new(move |view_id, _, _, _, _| self(view_id)))
    }
}

impl<F> IntoEventHandler<DropHandler, WithoutViewId> for F
where
    F: Fn(KeyState, MousePosition, &DragData, &mut DropEffect) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> DropHandler {
        DropHandler(Arc::new(
            move |_, key_state, mouse_position, drag_data, effect| {
                self(key_state, mouse_position, drag_data, effect)
            },
        ))
    }
}
