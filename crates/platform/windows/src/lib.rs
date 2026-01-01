use log::info;
use std::time::Duration;

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MsgWaitForMultipleObjectsEx, PeekMessageW, PostQuitMessage, TranslateMessage,
    MSG, MWMO_INPUTAVAILABLE, PM_REMOVE, QS_ALLINPUT,
};

mod conversions;
mod cursor;
#[cfg(feature = "drag-drop")]
mod drop_target;
#[cfg(feature = "monitor")]
mod monitor;
mod proc_handler;
#[cfg(feature = "tray")]
mod tray;
#[cfg(feature = "win7-compat")]
pub(crate) mod win7_compat;
mod window;
mod window_id;
mod window_proc;

#[cfg(feature = "monitor")]
pub use monitor::*;
#[cfg(feature = "tray")]
pub use tray::*;

pub mod base {
    pub use flor_platform_base::*;
}

pub mod events {
    use crate::base::HandleResult;
    use crate::base::Message;
    use crate::window_id::WindowId;

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
pub fn init() -> Result<(), Error> {
    #[cfg(feature = "drag-drop")]
    unsafe {
        use windows::Win32::System::Ole::OleInitialize;
        OleInitialize(None)?;
    }
    #[cfg(feature = "hi-dpi")]
    unsafe {
        #[cfg(not(feature = "win7-compat"))]
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
        #[cfg(feature = "win7-compat")]
        {
            use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;
            SetProcessDPIAware();
        }
    }
    Ok(())
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
