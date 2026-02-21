use flor_base::platform::WindowApi;
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
        title: String,
        width: u32,
        height: u32,
        sender: SyncSender<Result<WindowId, platform::Error>>,
    ) {
        self.queue.lock().push_back(PendingWindowRequest {
            title,
            width,
            height,
            result_sender: sender,
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
            let window_id = WindowId::create_window(&req.title, req.width, req.height);
            if req.result_sender.send(window_id).is_err() {
                // The Receiver must exist
                unreachable!()
            }
        }
    }
}

pub(crate) struct PendingWindowRequest {
    title: String,
    width: u32,
    height: u32,
    result_sender: SyncSender<Result<WindowId, platform::Error>>,
}
