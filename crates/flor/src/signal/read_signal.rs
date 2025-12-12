use std::marker::PhantomData;
use crate::signal::id::Id;
use crate::signal::read::Read;

/// 只读信号
#[derive(Copy, Clone)]
pub struct ReadSignal<T> {
    pub(crate) id: Id,
    pub(crate) _type: PhantomData<T>,
}

impl<T> Read<T> for ReadSignal<T> {
    fn id(&self) -> Id {
        self.id
    }
}
