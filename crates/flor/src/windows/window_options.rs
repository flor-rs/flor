use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::render::FlorRender;
use crate::signal::effect::updater_effect::create_updater;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use crate::windows::bus;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::WindowEntry;
use flor_graphics_base::Color;
use flor_platform_base::{WindowApi, WindowMode};
use log::trace;
use parking_lot::RwLock;
use platform::WindowId;

pub struct WindowOption {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub wait_v_sync: bool,
    pub continuous_rendering: bool,
    pub background_color: Color,
    pub view_fn: Option<Box<dyn Fn(WindowId) -> Box<dyn View + Send + Sync>>>,
}

impl Default for WindowOption {
    fn default() -> Self {
        Self {
            title: String::from("Window"),
            width: 800,
            height: 600,
            wait_v_sync: true,
            continuous_rendering: false,
            background_color: Color::rgb(255, 255, 255),
            view_fn: None,
        }
    }
}

impl WindowOption {
    pub fn open<F, V>(self, view_fn: F) -> Result<WindowId, Error>
    where
        F: Fn(WindowId) -> V + 'static,
        V: View + Send + Sync + 'static,
    {
        // 创建原生窗口
        let mut window_id = WindowId::create_window(&self.title, self.width, self.height)?;
        window_id.set_size((self.width, self.height))?;

        let (width, height) = window_id.get_client_size()?;
        window_id.set_window_mode(WindowMode::Normal)?;
        // 创建渲染器
        let render = FlorRender::create(window_id, width, height, self.wait_v_sync)?;

        let view_id = WindowEntry::new(window_id, self.continuous_rendering, self.background_color);

        VIEW_STORAGE
            .views
            .write()
            .insert(view_id, RwLock::new(Box::new(window_id)));

        let view_fn =
            Box::new(move |window_id| Box::new(view_fn(window_id)) as Box<dyn View + Send + Sync>);

        let root_dyn_view = create_updater(
            move || view_fn(window_id),
            move |view| {
                view_id.update_state(Box::new(view));
                VIEW_STORAGE.sweep_orphan_views();
            },
        );

        trace!("window root view: {:?}", root_dyn_view);
        VIEW_STORAGE.window_ids.write().insert(view_id, window_id);
        VIEW_STORAGE.add_child(view_id, root_dyn_view);
        window_id.bus_init_focus_manager_entry()?;

        bus::register_render(window_id, render);

        window_id.bus_create().error_on_err("on_create has error");
        window_id
            .bus_refresh_layout_entry()
            .expect("Failed re_layout_entry");
        window_id.update_window()?;
        Ok(window_id)
    }
}
