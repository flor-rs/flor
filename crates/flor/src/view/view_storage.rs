use crate::view::view_id::ViewId;
use crate::view::view_state::ViewState;
use crate::view::View;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use platform::WindowId;
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
    pub views: RwLock<SecondaryMap<ViewId, RwLock<Box<dyn View + Send + Sync+ 'static>>>>,
    /// 储存当前视图所在窗口id todo
    pub window_ids: RwLock<SecondaryMap<ViewId, WindowId>>,
    /// 储存父级关系
    pub parent_view_id: RwLock<SecondaryMap<ViewId, ViewId>>,
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

            self.parent_view_id.write().insert(child.view_id(),parent);
            if let Some(children) = child_ids.get_mut(parent) {
                children.push(child.view_id());
            } else {
                child_ids.insert(parent, vec![child.view_id()]);
            }
        }

        let child_view_id = child.view_id();

        self.views
            .write()
            .insert(child_view_id, RwLock::new(child));
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

    /// 移除视图及其所有子视图
    pub fn remove_view(&self, view_id: ViewId) {
        let children = {
            let child_ids = self.child_ids.read();
            child_ids.get(view_id).map(|c| c.clone())
        };

        // 先递归移除所有子视图
        if let Some(children) = children {
            for child in children {
                self.remove_view(child);
            }
        }

        // 只需要移除 view_id，其他关联数据会自动清理
        self.view_ids.lock().remove(view_id);
    }

    pub fn dispose_view(&self, view_id: ViewId) {
        // 1. 递归删除子控件
        let children = {
            let mut child_map = self.child_ids.write();
            child_map.remove(view_id)
        };

        if let Some(children) = children {
            for child_id in children {
                self.remove_view(child_id);
            }
        }

        // 2. 从父节点 child 列表中移除自己
        {
            let mut child_map = self.child_ids.write();
            for (_, children) in child_map.iter_mut() {
                children.retain(|&id| id != view_id);
            }
        }

        // 3. 删除 SecondaryMap 条目
        self.states.write().remove(view_id);
        self.views.write().remove(view_id);
        self.window_ids.write().remove(view_id);
        self.child_ids.write().remove(view_id); // 再清理一次防止残留

        // 4. 最后删除 SlotMap 中的 view_id
        self.view_ids.lock().remove(view_id);
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
