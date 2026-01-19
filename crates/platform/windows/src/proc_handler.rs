use crate::window_id::WindowId;
use flor_base::platform::HandleResult;
use flor_base::platform::Message;
use log::warn;
use once_cell::sync::Lazy;
use parking_lot::lock_api::RwLockReadGuard;
use parking_lot::{RawRwLock, RwLock};

// todo 考虑下panic？？？
pub static PROC_HANDLER: Lazy<RwLock<Box<dyn ProcHandler + Sync + Send>>> =
    Lazy::new(|| RwLock::new(Box::new(NoneProcHandler)));

pub fn set_proc_handler(proc_handler: Box<dyn ProcHandler + Sync + Send>) {
    let mut old_proc_handler = PROC_HANDLER.write();
    *old_proc_handler = proc_handler;
}

#[inline(always)]
pub fn proc() -> RwLockReadGuard<'static, RawRwLock, Box<dyn ProcHandler + Send + Sync>> {
    PROC_HANDLER.read()
}

pub trait ProcHandler {
    fn window_proc(&self, window_id: WindowId, message: Message) -> HandleResult;
}

pub struct NoneProcHandler;
impl ProcHandler for NoneProcHandler {
    fn window_proc(&self, window_id: WindowId, message: Message) -> HandleResult {
        warn!("ProcHandler::window_proc({:?}, {:?})", window_id, message);
        HandleResult::Default
    }
}

pub struct TestProcHandler;
impl ProcHandler for TestProcHandler {
    fn window_proc(&self, window_id: WindowId, message: Message) -> HandleResult {
        warn!("ProcHandler::window_proc({:?}, {:?})", window_id, message);
        HandleResult::Default
    }
}
