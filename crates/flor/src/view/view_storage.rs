use crate::view::view_id::ViewId;
use crate::view::view_state::ViewState;
use crate::view::View;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use platform::WindowId;
use rustc_hash::{FxHashMap, FxHashSet};
use slotmap::{SecondaryMap, SlotMap};
use std::fmt::Debug;
use std::sync::Arc;

/// 全局视图存储
/// 所有视图的状态都存储在这里，不再按窗口分类
pub static VIEW_STORAGE: Lazy<ViewStorage> = Lazy::new(|| ViewStorage::new());

#[derive(Debug)]
pub struct ViewStorage {
    /// 视图ID管理
    pub view_ids: Mutex<SlotMap<ViewId, ()>>,
    /// 视图状态存储
    pub states: RwLock<SecondaryMap<ViewId, RwLock<ViewState>>>,
    /// 父子关系存储
    pub child_ids: RwLock<SecondaryMap<ViewId, Vec<ViewId>>>,
    /// 视图树储存
    pub views: RwLock<SecondaryMap<ViewId, RwLock<Box<dyn View + Send + Sync + 'static>>>>,
    /// 储存当前视图所在窗口id todo
    pub window_ids: RwLock<SecondaryMap<ViewId, WindowId>>,
    /// 储存父级关系
    pub parent_view_id: RwLock<SecondaryMap<ViewId, ViewId>>,
    pub main_view_ids: RwLock<FxHashMap<WindowId, ViewId>>,
}

impl ViewStorage {
    fn new() -> Self {
        ViewStorage {
            view_ids: Default::default(),
            states: Default::default(),
            child_ids: Default::default(),
            views: Default::default(),
            window_ids: Default::default(),
            parent_view_id: Default::default(),
            main_view_ids: Default::default(),
        }
    }

    pub fn new_view(&self) -> ViewId {
        let mut view_ids = self.view_ids.lock();
        let view_id = view_ids.insert(());
        let mut states = self.states.write();
        states.insert(view_id, RwLock::new(ViewState::new()));
        view_id
    }

    /// 创建新视图
    pub fn new_view_with_state(&self, view_state: ViewState) -> ViewId {
        let mut view_ids = self.view_ids.lock();
        let view_id = view_ids.insert(());
        let mut states = self.states.write();
        states.insert(view_id, RwLock::new(view_state));
        view_id
    }

    /// 添加子视图
    pub fn add_child(&self, parent: ViewId, child: Box<dyn View + Send + Sync>) {
        {
            let mut child_ids = self.child_ids.write();

            self.parent_view_id.write().insert(child.view_id(), parent);
            if let Some(children) = child_ids.get_mut(parent) {
                children.push(child.view_id());
            } else {
                child_ids.insert(parent, vec![child.view_id()]);
            }
        }

        let child_view_id = child.view_id();

        self.views.write().insert(child_view_id, RwLock::new(child));
        if let Some(view_state) = VIEW_STORAGE.states.read().get(parent) {
            view_state.write().dirty_children = true;
        }

        // 关联窗口检索
        let x = { VIEW_STORAGE.window_ids.read().get(parent).cloned() };
        if let Some(window_id) = x {
            Self::set_all_child_window_id(child_view_id, window_id);
        }
    }

    fn set_all_child_window_id(view_id: ViewId, window_id: WindowId) {
        {
            VIEW_STORAGE.window_ids.write().insert(view_id, window_id);
        }
        let child_ids = VIEW_STORAGE.child_ids.read();
        if let Some(child_ids) = child_ids.get(view_id).cloned() {
            for child_id in child_ids {
                Self::set_all_child_window_id(child_id, window_id);
            }
        }
    }

    pub fn dispose_view(&self, view_id: ViewId) {
        if !self.states.read().contains_key(view_id) {
            return;
        }

        let children = {
            let mut child_ids = self.child_ids.write();
            child_ids.remove(view_id)
        };

        if let Some(children) = children {
            for child in children {
                self.dispose_view(child);
            }
        }

        if let Some(parent) = view_id.parent_view_id() {
            {
                let mut child_ids = self.child_ids.write();
                if let Some(children) = child_ids.get_mut(parent) {
                    children.retain(|&id| id != view_id);
                }
            }

            if let Some(parent_state) = self.states.write().get(parent) {
                parent_state.write().dirty_children = true;
            }
        }

        self.states.write().remove(view_id);
        self.views.write().remove(view_id);
        self.window_ids.write().remove(view_id);

        // 6. 最后移除 ID
        self.view_ids.lock().remove(view_id);
    }

    pub fn sweep_orphan_views(&self) {
        use rustc_hash::FxHashSet;

        let all_views = self.states.read().keys().collect::<FxHashSet<ViewId>>();

        let mut alive = FxHashSet::default();

        {
            let child_ids = self.child_ids.read();
            let main_roots = self.main_view_ids.read();

            fn mark(
                view_id: ViewId,
                child_ids: &SecondaryMap<ViewId, Vec<ViewId>>,
                alive: &mut FxHashSet<ViewId>,
            ) {
                if !alive.insert(view_id) {
                    return;
                }

                if let Some(children) = child_ids.get(view_id) {
                    for &child in children {
                        mark(child, child_ids, alive);
                    }
                }
            }

            // --- 核心修改点：遍历所有窗口的根节点 ---
            for &root_id in main_roots.values() {
                if all_views.contains(&root_id) {
                    mark(root_id, &child_ids, &mut alive);
                }
            }
        }

        let dead_views = all_views.difference(&alive).copied().collect::<Vec<ViewId>>();

        if !dead_views.is_empty() {
            for dead_view_id in dead_views {
                self.dispose_view(dead_view_id);
            }
        }
    }
}

impl Default for ViewStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Box<dyn View + Send + Sync> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("view_id", &self.view_id())
            .finish()
    }
}
