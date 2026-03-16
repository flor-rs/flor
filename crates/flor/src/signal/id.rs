use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    Signal(usize),
    Effect(usize),
    List(usize),
}

impl Id {
    fn next() -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn next_signal_id() -> Self {
        Self::Signal(Self::next())
    }

    pub fn next_effect_id() -> Self {
        Self::Effect(Self::next())
    }

    pub fn next_list_id() -> Self {
        Self::List(Self::next())
    }
}
