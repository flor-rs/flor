use crate::error::Error;
use crate::view::View;
use crate::windows::window_options::WindowOption;
use parking_lot::Mutex;
use platform::WindowId;
use std::collections::VecDeque;
use std::sync::mpsc::SyncSender;

pub struct WindowCreationQueue {
    queue: Mutex<VecDeque<PendingWindowRequest>>,
}

impl WindowCreationQueue {
    pub const fn new() -> WindowCreationQueue {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn pending_window(
        &self,
        window_option: WindowOption,
        view_fn: Box<dyn Fn(WindowId) -> Vec<Box<dyn View + Send + Sync + 'static>> + Send>,
        result_sender: SyncSender<Result<WindowId, Error>>,
    ) {
        self.queue.lock().push_back(PendingWindowRequest {
            window_option,
            view_fn,
            result_sender,
        });
    }

    pub fn try_spawn(&self) {
        let tasks = {
            let mut queue = self.queue.lock();
            if queue.is_empty() {
                return;
            }
            std::mem::take(&mut *queue)
        };

        for req in tasks {
            let window_id = req.window_option.open_with_box(req.view_fn);
            if req.result_sender.send(window_id).is_err() {
                // The Receiver must exist
                unreachable!()
            }
        }
    }
}

pub(crate) struct PendingWindowRequest {
    window_option: WindowOption,
    view_fn: Box<dyn Fn(WindowId) -> Vec<Box<dyn View + Send + Sync + 'static>> + Send>,
    result_sender: SyncSender<Result<WindowId, Error>>,
}
