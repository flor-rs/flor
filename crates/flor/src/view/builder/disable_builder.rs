use crate::signal::create_updater_with_id;
use crate::view::ViewIdentity;

pub trait DisableBuilder {
    fn disable<F>(self, f: F) -> Self
    where
        F: Fn() -> bool + 'static;
}

impl<V: ViewIdentity> DisableBuilder for V {
    fn disable<F>(self, f: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        let view_id = self.identity();
        let val = create_updater_with_id(
            move || f(),
            move |disable| {
                let _ = view_id.with_state_mut(|x| {
                    x.disable = disable;
                });
            },
        );
        view_id.pending_effect_id(val.0);
        self
    }
}
