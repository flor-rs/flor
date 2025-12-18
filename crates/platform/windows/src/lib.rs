use log::info;
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MsgWaitForMultipleObjectsEx, PeekMessageW, PostQuitMessage, TranslateMessage,
    MSG, MWMO_INPUTAVAILABLE, PM_REMOVE, QS_ALLINPUT,
};

mod conversions;
mod cursor;
mod drop_target;
#[cfg(feature = "monitor")]
mod monitor;
mod proc_handler;
mod window;
mod window_id;
mod window_proc;

#[cfg(feature = "monitor")]
pub use monitor::*;

pub mod base {
    pub use flor_platform_base::*;
}

pub mod events {
    use crate::window_id::WindowId;
    use flor_platform_base::HandleResult;
    use flor_platform_base::Message;

    pub type EventMessageHandler = fn(window_id: WindowId, message: Message) -> HandleResult;
}

pub use {proc_handler::*, window_id::WindowId, windows::core::Error};

#[inline]
pub fn handler_wait(timeout: Option<Duration>) {
    info!("lazy wait: {:?}", timeout);
    let millis = match timeout {
        None => u32::MAX, // 无限等待，对应纯静态
        Some(d) => d.as_millis().min(u32::MAX as u128) as u32,
    };
    unsafe {
        MsgWaitForMultipleObjectsEx(None, millis, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    }
}

#[inline]
pub fn handler_message() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

pub fn exit() {
    unsafe {
        PostQuitMessage(0);
    }
}
