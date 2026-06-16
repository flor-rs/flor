use crate::error::Error;
use crate::render::FlorRenderer;
use crate::signal::create_updater;
use crate::view::builder::ViewBuilder;
use crate::view::resolver::UnitMetrics;
use crate::view::{IntoViewIter, View};
use crate::view::{ViewStorage, VIEW_STORAGE};
use crate::windows::bus;
use crate::windows::WindowBusDispatchEntry;
use crate::windows::WindowEntry;
use arc_swap::ArcSwap;
use flor_base::platform::{WindowApi, WindowMode};
use flor_base::types::Color;
use log::trace;
use parking_lot::RwLock;
use platform::WindowId;
use std::sync::Arc;
use std::time::Duration;

pub struct WindowOption {
    pub title: String,
    pub width: u32,
    pub height: u32,
    /// Pixel size of 1rem for layout and class length resolution in this window.
    ///
    /// This value seeds `UnitMetrics::rem_px`. The default is 16.0.
    pub rem_px: f32,
    pub wait_v_sync: bool,
    pub show_fps: bool,
    pub continuous_rendering: bool,
    pub background_color: Color,
    pub tooltip_delay: Duration,
}

impl Default for WindowOption {
    fn default() -> Self {
        Self {
            title: String::from("Window"),
            width: 800,
            height: 600,
            rem_px: 16.0,
            wait_v_sync: true,
            continuous_rendering: false,
            background_color: Color::rgb(255, 255, 255),
            show_fps: false,
            tooltip_delay: Duration::from_millis(500),
        }
    }
}

impl WindowOption {
    /// 创建并显示窗口。
    ///
    /// 支持从任意线程调用：
    /// - 主线程：直接创建
    /// - 子线程：自动投递到主线程执行，当前线程阻塞等待结果
    pub fn open<F, V>(self, view_fn: F) -> Result<WindowId, Error>
    where
        F: Fn(WindowId) -> V + Send + Sync + 'static,
        V: IntoViewIter,
    {
        let view_fn =
            Box::new(move |window_id| view_fn(window_id).into_view_iter().collect::<Vec<_>>());

        #[cfg(feature = "cross-thread-window-creation")]
        {
            // 如果已在主线程，直接短路返回，不进入投递流程
            if platform::is_event_loop_thread() {
                return self.open_with_box(view_fn);
            }
            use std::sync::mpsc;
            let (tx, rx) = mpsc::sync_channel(1);
            crate::WINDOW_SPAWNER.pending_window(self, view_fn, tx);
            platform::wake_event_loop();

            return rx.recv().map_err(|_| {
                Error::InitError("The event loop has ended, but the window creation request has not been processed.".into())
            })?;
        }

        #[cfg(not(feature = "cross-thread-window-creation"))]
        self.open_with_box(view_fn)
    }

    pub fn open_with_box(
        self,
        view_fn: Box<dyn Fn(WindowId) -> Vec<Box<dyn View + Send + Sync + 'static>>>,
    ) -> Result<WindowId, Error> {
        let window_id = WindowId::create_window(&self.title, self.width, self.height)?;

        window_id.set_size((self.width, self.height))?;

        let (width, height) = window_id.get_client_size()?;
        window_id.set_window_mode(WindowMode::Normal)?;

        // 创建渲染器
        let render = FlorRenderer::create(window_id, width, height, self.wait_v_sync)?;
        bus::register_render(window_id, render);

        let (dpi_x, dpi_y) = window_id.get_dpi()?;
        let (w, h) = window_id.get_client_size()?;
        let unit = Arc::new(ArcSwap::from_pointee(UnitMetrics::new(
            dpi_x,
            dpi_y,
            self.rem_px,
            w as f32,
            h as f32,
        )));
        let view_id = WindowEntry::new(
            window_id,
            self.continuous_rendering,
            self.show_fps,
            self.background_color,
            unit.clone(),
            self.tooltip_delay,
        );

        VIEW_STORAGE
            .views
            .write()
            .insert(view_id, RwLock::new(Box::new(window_id)));

        let root_dyn_view = create_updater(
            move || view_fn(window_id),
            move |view| {
                view_id.update_state(Box::new(view));
                VIEW_STORAGE.sweep_orphan_views();
            },
        );

        trace!("window root view: {:?}", root_dyn_view);
        VIEW_STORAGE.window_ids.write().insert(view_id, window_id);
        window_id.views(root_dyn_view);

        ViewStorage::init_window_child(view_id, window_id)?;

        window_id.bus_init_focus_manager_entry()?;
        window_id.bus_create_entry()?;
        window_id.bus_refresh_layout_entry()?;
        window_id.update_window()?;
        Ok(window_id)
    }
}
