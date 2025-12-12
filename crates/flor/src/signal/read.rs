use crate::signal::id::Id;
use crate::signal::runtime::{RUNTIME, SCOPE};

pub trait Read<T> {
    fn id(&self) -> Id;

    /// 跟踪
    fn track(&self) {
        if let Some(scope_id) = SCOPE.get() {
            RUNTIME.subscribe(self.id(), scope_id);
        }
    }

    fn try_get(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.track();
        RUNTIME.values.get(&self.id())?.get::<T>()
    }

    fn get(&self) -> T
    where
        T: Clone + 'static,
    {
        self.track();
        RUNTIME
            .values
            .get(&self.id())
            .expect("invalid signal id")
            .get::<T>()
            .expect("to downcast signal type fail")
            .clone()
    }

    fn destroy(&self) {
        RUNTIME.destroy_signal(self.id());
    }
}
