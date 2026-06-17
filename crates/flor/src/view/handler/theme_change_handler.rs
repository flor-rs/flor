use crate::view::handler::{IntoEventHandler, NoArgs, ViewIdOnly, WithoutViewId};
use crate::view::ViewId;
use flor_base::platform::ThemeMode;
use std::sync::Arc;

#[derive(Clone)]
pub struct OnThemeChangedHandler(pub Arc<dyn Fn(ViewId, ThemeMode) + Send + Sync + 'static>);

impl<F> From<F> for OnThemeChangedHandler
where
    F: Fn(ViewId, ThemeMode) + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        OnThemeChangedHandler(Arc::new(f))
    }
}

impl<F> IntoEventHandler<OnThemeChangedHandler, NoArgs> for F
where
    F: Fn() + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnThemeChangedHandler {
        OnThemeChangedHandler(Arc::new(move |_, _| self()))
    }
}

impl<F> IntoEventHandler<OnThemeChangedHandler, ViewIdOnly> for F
where
    F: Fn(ViewId) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnThemeChangedHandler {
        OnThemeChangedHandler(Arc::new(move |view_id, _| self(view_id)))
    }
}

impl<F> IntoEventHandler<OnThemeChangedHandler, WithoutViewId> for F
where
    F: Fn(ThemeMode) + Send + Sync + 'static,
{
    fn into_event_handler(self) -> OnThemeChangedHandler {
        OnThemeChangedHandler(Arc::new(move |_, theme_mode| self(theme_mode)))
    }
}
