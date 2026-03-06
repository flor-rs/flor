#[cfg(feature = "direct2d")]
pub extern crate flor_graphics_direct2d as graphics_gpu;
#[cfg(feature = "opengl")]
pub extern crate flor_graphics_opengl as graphics_gpu;
#[cfg(feature = "tiny-skia")]
pub extern crate flor_graphics_tiny_skia as graphics_cpu;
pub extern crate flor_platform as platform;
pub extern crate once_cell;
pub extern crate parking_lot;
pub extern crate rustc_hash;

pub mod graphics {
    pub mod base {
        pub use flor_base::graphics::*;
    }
}

pub mod device_kind;
pub mod error;
pub mod log_error;
mod min_wait_time;
pub mod proc;
pub mod render;
pub mod signal;
pub mod view;
pub mod windows;

pub type ComputedLayout = taffy::Layout;

use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::min_wait_time::MinWaitTime;
use crate::proc::WindowsProcHandler;
use crate::signal::effect::updater_effect::create_updater;
use crate::windows::bus::RENDERS;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::WINDOW_ENTRY_MAP;
pub use flor_base::types;
use log::{debug, info, trace};
use once_cell::sync::Lazy;
use platform::set_proc_handler;
#[cfg(feature = "tray")]
use platform::{base::TrayEvent, base::TrayManagerEntry, base::TrayOptions, Tray, TrayId};
pub use slotmap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
pub use taffy;

static ALLOW_NO_WINDOWS_LOOP: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static EXIT: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

pub static CONFIG: Lazy<bool> = Lazy::new(|| false);

#[cfg(feature = "cross-thread-window-creation")]
use crate::windows::window_creation_queue::WindowCreationQueue;
#[cfg(feature = "cross-thread-window-creation")]
static WINDOW_SPAWNER: WindowCreationQueue = WindowCreationQueue::new();

pub struct FlorGui;

impl FlorGui {
    #[inline]
    pub fn init(&self) -> Result<(), Error> {
        set_proc_handler(Box::new(WindowsProcHandler::default()));
        #[cfg(feature = "tray")]
        Tray::init()?;
        Ok(())
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
        platform::exit();
        EXIT.store(true, Ordering::Release);
    }

    pub fn event_loop(&self) -> Result<(), Error> {
        #[cfg(feature = "cross-thread-window-creation")]
        // 记录事件循环线程
        platform::record_event_loop_thread();

        // 即使没有窗口，也要处理可能已排队的跨线程创建请求
        #[cfg(feature = "cross-thread-window-creation")]
        WINDOW_SPAWNER.try_spawn();

        if RENDERS.is_empty() {
            return Ok(());
        }

        platform::init()?;

        let mut last_fps_time = Instant::now();
        let mut frame_count = 0;
        let mut last_frame_count = 0;

        loop {
            platform::handler_message();

            // 处理跨线程窗口创建请求
            #[cfg(feature = "cross-thread-window-creation")]
            WINDOW_SPAWNER.try_spawn();

            let allow = ALLOW_NO_WINDOWS_LOOP.load(Ordering::Acquire);
            trace!("allow_no_windows_loop: {}", allow);

            if (!allow && RENDERS.is_empty()) || EXIT.load(Ordering::Acquire) {
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

                // tooltip: 检查悬停计时器，合并等待时间
                let tooltip_wait = window_id.bus_tooltip_check_entry();
                min_wait_time.update_to_min_wait_time(tooltip_wait);
            }

            for window_id in re_layout_window_ids {
                window_id
                    .bus_refresh_layout_entry()
                    .error_on_err("bus refresh layout error")
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
