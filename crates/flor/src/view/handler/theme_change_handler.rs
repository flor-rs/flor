use crate::view::view_id::ViewId;
use flor_platform_base::ThemeMode;
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
