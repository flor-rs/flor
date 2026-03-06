use log::info;
#[cfg(feature = "cross-thread-window-creation")]
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
#[cfg(feature = "cross-thread-window-creation")]
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{PostThreadMessageW, WM_USER},
};

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MsgWaitForMultipleObjectsEx, PeekMessageW, PostQuitMessage, TranslateMessage,
    MSG, MWMO_INPUTAVAILABLE, PM_REMOVE, QS_ALLINPUT,
};
#[cfg(feature = "clipboard")]
pub mod clipboard;
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
    pub use flor_base::platform::*;
}

pub mod events {
    use crate::base::HandleResult;
    use crate::base::Message;
    use crate::window_id::WindowId;

    pub type EventMessageHandler = fn(window_id: WindowId, message: Message) -> HandleResult;
}

pub use {proc_handler::*, window_id::WindowId, windows::core::Error};

/// 事件循环所在线程的 Win32 线程 ID
#[cfg(feature = "cross-thread-window-creation")]
static EVENT_LOOP_THREAD_ID: AtomicU32 = AtomicU32::new(0);

/// 在事件循环启动时调用，记录当前线程为事件循环线程。
///
/// 必须在 `event_loop()` 入口处调用一次。
#[cfg(feature = "cross-thread-window-creation")]
pub fn record_event_loop_thread() {
    let thread_id = unsafe { GetCurrentThreadId() };
    EVENT_LOOP_THREAD_ID.store(thread_id, Ordering::Release);
}

#[cfg(feature = "cross-thread-window-creation")]
pub fn is_event_loop_thread() -> bool {
    let thread_id = unsafe { GetCurrentThreadId() };
    let event_loop_thread_id = EVENT_LOOP_THREAD_ID.load(Ordering::Acquire);
    if event_loop_thread_id == 0 {
        return true;
    }
    event_loop_thread_id == thread_id
}

/// 从任意线程唤醒事件循环。
///
/// 向事件循环线程的消息队列投递一条空消息，
/// 使 `MsgWaitForMultipleObjectsEx` 立即返回。
#[cfg(feature = "cross-thread-window-creation")]
pub fn wake_event_loop() {
    let tid = EVENT_LOOP_THREAD_ID.load(Ordering::Acquire);
    if tid != 0 {
        unsafe {
            let _ = PostThreadMessageW(tid, WM_USER, WPARAM(0), LPARAM(0));
        }
    }
}

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
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
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
