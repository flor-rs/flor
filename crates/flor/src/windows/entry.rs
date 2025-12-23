use crate::view::focus_manager::FocusManager;
use crate::view::view_id::ViewId;
use dashmap::mapref::one::{Ref, RefMut};
use dashmap::DashMap;
use flor_graphics_base::Color;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use platform::WindowId;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use taffy::TaffyTree;

/// 为window_id 和分配 view_id 建立映射
pub(crate) static WINDOW_ENTRY_MAP: Lazy<DashMap<WindowId, WindowEntry>> =
    Lazy::new(|| DashMap::new());

/// 储存window的各种非view的东西
pub struct WindowEntry {
    pub view_id: ViewId,
    pub taffy_tree: RwLock<TaffyTree<ViewId>>,
    // If the mouse moves out of the window, the IDs below will be set to None.
    pub hover_id: Option<ViewId>,
    pub focus_manager: FocusManager,
    pub continuous_rendering: bool,
    pub fps: AtomicU32,
    pub show_fps: bool,
    pub layout_dirty: AtomicBool,
    pub l_down_view_id: Option<ViewId>,
    pub r_down_view_id: Option<ViewId>,
    pub m_down_view_id: Option<ViewId>,
    pub capture_view_id: Option<ViewId>,
    pub current_drag_target: Option<ViewId>,
    pub background_color: Color,
}

impl WindowEntry {
    pub fn new(window_id: WindowId, continuous_rendering: bool, background_color: Color) -> ViewId {
        let view_id = ViewId::new();

        let window_entry = Self {
            view_id,
            taffy_tree: RwLock::new(TaffyTree::new()),
            hover_id: None,
            focus_manager: Default::default(),
            continuous_rendering,
            fps: AtomicU32::new(0),
            show_fps: false,
            layout_dirty: AtomicBool::new(false),
            l_down_view_id: None,
            r_down_view_id: None,
            m_down_view_id: None,
            capture_view_id: None,
            current_drag_target: None,
            background_color,
        };
        WINDOW_ENTRY_MAP.insert(window_id, window_entry);
        view_id
    }

    pub fn is_continuous_rendering(&self) -> bool {
        self.continuous_rendering
    }
    pub fn set_continuous_rendering(&mut self, enable: bool) {
        self.continuous_rendering = enable;
    }
    pub fn is_show_fps(&self) -> bool {
        self.show_fps
    }
    pub fn set_show_fps(&mut self, enable: bool) {
        self.show_fps = enable;
    }

    pub fn is_layout_dirty(&self) -> bool {
        self.layout_dirty.load(Ordering::Acquire)
    }
    pub fn mark_layout_dirty(&self) {
        self.layout_dirty.store(true, Ordering::Release);
    }
    pub fn clear_layout_dirty(&self) {
        self.layout_dirty.store(false, Ordering::Release);
    }
}

pub trait WindowEntryVisit {
    fn entry(&'_ self) -> Option<Ref<'_, WindowId, WindowEntry>>;
    fn entry_mut(&'_ self) -> Option<RefMut<'_, WindowId, WindowEntry>>;
}

impl WindowEntryVisit for WindowId {
    fn entry(&'_ self) -> Option<Ref<'_, WindowId, WindowEntry>> {
        WINDOW_ENTRY_MAP.get(self)
    }
    fn entry_mut(&'_ self) -> Option<RefMut<'_, WindowId, WindowEntry>> {
        WINDOW_ENTRY_MAP.get_mut(self)
    }
}
