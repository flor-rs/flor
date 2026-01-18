use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(usize);

pub type EffectId = Id;

impl Id {
    pub(crate) fn next() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Id(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}