use crate::signal::{Id, Read, Signal, SignalRef};

#[derive(Debug, Clone)]
pub struct ConstSignal<T: Clone> {
    value: T,
}

impl<T: Clone> ConstSignal<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Clone> Signal for ConstSignal<T> {
    fn id(&self) -> Id {
        Id::Const
    }

    fn exists(&self) -> bool {
        true
    }

    fn destroy(&self) {}
}

impl<T: Clone> Read<T> for ConstSignal<T> {
    fn track(&self) {}
    fn try_get(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        Some(self.value.clone())
    }
    fn get(&self) -> T
    where
        T: Clone + 'static,
    {
        self.value.clone()
    }

    fn get_ref(&self) -> SignalRef<'_, T> {
        SignalRef::constant(&self.value)
    }
    fn try_get_ref(&self) -> Option<SignalRef<'_, T>> {
        Some(self.get_ref())
    }
}
