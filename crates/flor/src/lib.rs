#[cfg(feature = "direct2d")]
pub extern crate flor_graphics_direct2d as graphics;
#[cfg(windows)]
pub extern crate flor_platform_windows as platform;
pub extern crate once_cell;
pub extern crate parking_lot;
pub extern crate rustc_hash;

use crate::proc::WindowsProcHandler;
use crate::windows::bus::RENDERS;
use log::{debug, info, trace};
use once_cell::sync::Lazy;
use platform::set_proc_handler;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "clipboard")]
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub mod device_kind;
pub mod error;
pub mod log_error;
mod min_wait_time;
pub mod proc;
pub mod render;
pub mod signal;
pub mod view;
pub mod windows;

use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::min_wait_time::MinWaitTime;
use crate::signal::effect::updater_effect::create_updater;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::WINDOW_ENTRY_MAP;
#[cfg(feature = "clipboard")]
pub use arboard;
#[cfg(feature = "tray")]
use platform::{base::TrayEvent, base::TrayManagerEntry, base::TrayOptions, Tray, TrayId};
pub use slotmap;
pub use taffy;

static ALLOW_NO_WINDOWS_LOOP: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static EXIT: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

pub static CONFIG: Lazy<bool> = Lazy::new(|| false);
#[cfg(feature = "clipboard")]
static CLIPBOARD: OnceLock<arboard::Clipboard> = OnceLock::new();

pub struct FlorGui;

impl FlorGui {
    #[inline]
    pub fn init(&self) -> Result<(), Error> {
        set_proc_handler(Box::new(WindowsProcHandler::default()));
        #[cfg(feature = "clipboard")]
        let _ = CLIPBOARD.set(arboard::Clipboard::new()?);
        #[cfg(feature = "tray")]
        Tray::init()?;
        Ok(())
    }

    #[cfg(feature = "clipboard")]
    pub fn clipboard<'a>(&self) -> &'a arboard::Clipboard {
        CLIPBOARD.get().expect("Clipboard not initialized")
    }

    #[cfg(feature = "tray")]
    pub fn tray_add(&self, options: &TrayOptions) -> Result<TrayId, Error> {
        let tray_id = Tray::add(options)?;
        Ok(tray_id)
    }

    #[cfg(feature = "tray")]
    pub fn tray_update(&self, tray_id: TrayId, options: &TrayOptions) -> Result<(), Error> {
        Tray::update(tray_id, options)?;
        Ok(())
    }

    #[cfg(feature = "tray")]
    pub fn tray_remove(&self, tray_id: TrayId) -> Result<(), Error> {
        Tray::remove(tray_id)?;
        Ok(())
    }

    #[cfg(feature = "tray")]
    /// 本质上是set
    pub fn tray_on_callback(&self, f: impl Fn(TrayId, TrayEvent) + Send + Sync + 'static) {
        Tray::on_callback(f);
    }

    pub fn exit(&self) {
        if ALLOW_NO_WINDOWS_LOOP.load(Ordering::Acquire) {
            platform::exit();
            EXIT.store(true, Ordering::Release);
        }
    }

    pub fn event_loop(&self) -> Result<(), Error> {
        if RENDERS.is_empty() {
            return Ok(());
        }

        platform::init()?;

        let mut last_fps_time = Instant::now();
        let mut frame_count = 0;
        let mut last_frame_count = 0;

        loop {
            platform::handler_message();

            let allow = ALLOW_NO_WINDOWS_LOOP.load(Ordering::Acquire);
            trace!("allow_no_windows_loop: {}", allow);

            if !allow && EXIT.load(Ordering::Acquire) {
                info!("application exit.");
                break Ok(());
            }

            signal::runtime::RUNTIME.execute_update_queue();

            let mut lazy = true;
            let mut re_draw_window_ids = vec![];
            let mut re_layout_window_ids = vec![];
            let mut all_window_ids = vec![];
            for entry in WINDOW_ENTRY_MAP.iter() {
                let window_id = *entry.key();
                all_window_ids.push(window_id);
                if entry.is_layout_dirty() {
                    re_layout_window_ids.push(window_id);
                }
                if !entry.continuous_rendering {
                    entry.fps.store(-1, Ordering::Release);
                    continue;
                } else {
                    entry.fps.store(last_frame_count, Ordering::Release);
                }
                re_draw_window_ids.push(window_id);
                lazy = false;
            }

            let mut min_wait_time = None;
            for window_id in &all_window_ids {
                let child_wait_time = window_id.bus_frame_entry();

                if child_wait_time.is_err() {
                    child_wait_time.error_on_err("bus frame entry error");
                    continue;
                }

                let Ok(child_wait_time) = child_wait_time else {
                    unreachable!()
                };

                min_wait_time.update_to_min_wait_time(child_wait_time);
            }

            for window_id in re_layout_window_ids {
                window_id
                    .bus_refresh_layout_entry()
                    .error_on_err("bus refresh layout error")
            }

            // 布局之后，进行可视检测
            for window_id in &all_window_ids {
                window_id.bus_visual_test_entry();
            }

            for window_id in re_draw_window_ids {
                window_id
                    .bus_re_draw_entry()
                    .error_on_err("bus draw entry error");
            }

            // --- 新增 FPS 统计逻辑 ---
            frame_count += 1;
            let elapsed = last_fps_time.elapsed();
            if elapsed >= Duration::from_millis(1000) {
                let fps = frame_count as f64 / elapsed.as_secs_f64(); // 计算 帧数/时间
                debug!(
                    "Current FPS: {:.2} (Frames: {}, Time: {:.2}s)",
                    fps,
                    frame_count,
                    elapsed.as_secs_f64()
                );

                last_frame_count = frame_count;
                frame_count = 0;
                last_fps_time = Instant::now();
            }

            if lazy {
                platform::handler_wait(min_wait_time);
            }
        }
    }

    pub fn allow_no_windows_loop(&self, allow_no_windows_loop: impl Fn() -> bool + 'static) {
        create_updater(
            move || allow_no_windows_loop(),
            |value| ALLOW_NO_WINDOWS_LOOP.store(value, Ordering::Release),
        );
    }
}
