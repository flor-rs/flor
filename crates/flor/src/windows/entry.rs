use crate::view::focus_manager::FocusManager;
use crate::view::resolver::Unit;
use crate::view::ViewId;
use arc_swap::ArcSwap;
use dashmap::mapref::one::{Ref, RefMut};
use dashmap::DashMap;
use flor_base::platform::{KeyState, MousePosition};
use flor_base::types::Color;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use platform::WindowId;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    pub fps: AtomicI32,
    pub show_fps: bool,
    pub layout_dirty: AtomicBool,
    pub l_down_view_id: Option<ViewId>,
    pub r_down_view_id: Option<ViewId>,
    pub m_down_view_id: Option<ViewId>,
    pub capture_view_id: Option<ViewId>,
    pub current_drag_target: Option<ViewId>,
    pub background_color: Color,
    pub unit: Arc<ArcSwap<Unit>>,
    /// tooltip 计时起点：鼠标进入当前 hover 控件时的时间戳
    pub tooltip_hover_start: Option<Instant>,
    /// 已经触发了 tooltip_show 的控件（避免重复触发）
    pub tooltip_shown_for: Option<ViewId>,
    /// tooltip 显示延迟
    pub tooltip_delay: Duration,
    /// 缓存的最后一次鼠标窗口坐标（供 tooltip 帧回调使用）
    pub last_mouse_position: MousePosition,
    /// 缓存的最后一次键盘状态（供 tooltip 帧回调使用）
    pub last_key_state: KeyState,
}

impl WindowEntry {
    pub fn new(
        window_id: WindowId,
        continuous_rendering: bool,
        show_fps: bool,
        background_color: Color,
        unit: Arc<ArcSwap<Unit>>,
        tooltip_delay: Duration,
    ) -> ViewId {
        let view_id = ViewId::new();

        let window_entry = Self {
            view_id,
            taffy_tree: RwLock::new(TaffyTree::new()),
            hover_id: None,
            focus_manager: Default::default(),
            continuous_rendering,
            fps: AtomicI32::new(-1),
            show_fps,
            layout_dirty: AtomicBool::new(false),
            l_down_view_id: None,
            r_down_view_id: None,
            m_down_view_id: None,
            capture_view_id: None,
            current_drag_target: None,
            background_color,
            unit,
            tooltip_hover_start: None,
            tooltip_shown_for: None,
            tooltip_delay,
            last_mouse_position: MousePosition { x: 0, y: 0 },
            last_key_state: KeyState::default(),
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
