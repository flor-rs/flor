use std::time::Duration;

pub trait MinWaitTime {
    fn update_to_min_wait_time(&mut self, other: Option<Duration>);
}

impl MinWaitTime for Option<Duration> {
    fn update_to_min_wait_time(&mut self, other: Option<Duration>) {
        *self = match (*self, other) {
            (None, None) => None,
            (None, Some(t)) => Some(t),
            (Some(t), None) => Some(t),
            (Some(t1), Some(t2)) => Some(t1.min(t2)),
        }
    }
}
