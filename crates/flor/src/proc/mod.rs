use crate::windows::bus;
use log::error;
use platform::base::Message;
use platform::base::HandleResult;
use platform::ProcHandler;
use platform::WindowId;

#[derive(Debug, Default)]
pub struct WindowsProcHandler {}
impl ProcHandler for WindowsProcHandler {
    fn window_proc(&self, window_id: WindowId, message: Message) -> HandleResult {
        // trace!("window_proc({:?}, {:?})", window_id, message);
        bus::event(window_id, message).unwrap_or_else(|err| {
            error!("bus error: {:?}", err);
            HandleResult::Default
        })
    }
}
