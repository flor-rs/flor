use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::signal::{Id, RUNTIME};
use crate::view::handler::ViewHandler;
use crate::view::{ScrollState, View, ViewId, ViewState};
use crate::windows::WINDOW_ENTRY_MAP;
use flor_base::types::{Rect, Transform2D};
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use platform::WindowId;
use slotmap::{Key, SecondaryMap, SlotMap};
use std::fmt::Debug;

/// 全局视图存储
/// 所有视图的状态都存储在这里，不再按窗口分类
pub static VIEW_STORAGE: Lazy<ViewStorage> = Lazy::new(|| ViewStorage::new());

/// 对于不是每个view都要求有值的数据，应该优先增加map而不是放到state里。
#[derive(Debug)]
pub struct ViewStorage {
    /// 视图ID管理
    pub view_ids: Mutex<SlotMap<ViewId, ()>>,
    /// 视图状态存储
    pub states: RwLock<SecondaryMap<ViewId, RwLock<ViewState>>>,
    pub handlers: RwLock<SecondaryMap<ViewId, RwLock<ViewHandler>>>,
    /// 父子关系存储，总是根据z-index排序的
    pub child_ids: RwLock<SecondaryMap<ViewId, Vec<ViewId>>>,
    /// 视图树储存
    pub views: RwLock<SecondaryMap<ViewId, RwLock<Box<dyn View + Send + Sync + 'static>>>>,
    /// 储存当前视图所在窗口id
    pub window_ids: RwLock<SecondaryMap<ViewId, WindowId>>,
    /// 储存父级关系
    pub parent_view_id: RwLock<SecondaryMap<ViewId, ViewId>>,
    pub focus_index: RwLock<SecondaryMap<ViewId, u32>>,
    pub focus_scope: RwLock<SecondaryMap<ViewId, u32>>,
    pub pressed: RwLock<SecondaryMap<ViewId, ()>>,
    pub visual: RwLock<SecondaryMap<ViewId, ()>>,
    pub scroll: RwLock<SecondaryMap<ViewId, ScrollState>>,
    pub view_z_index: RwLock<SecondaryMap<ViewId, i32>>,
    pub pending_effect_id: RwLock<SecondaryMap<ViewId, Vec<Id>>>,
    // 设置的 变换 ，这个在绘制时需要用到，在布局时，需要用来计算
    pub transform: RwLock<SecondaryMap<ViewId, Transform2D>>,
    // 当前控件累积的 变换 ，这个用来处理命中测试，在布局时计算
    pub accumulated_transform: RwLock<SecondaryMap<ViewId, Transform2D>>,
    // 缓存的视觉矩形（布局后计算）
    pub visual_rect: RwLock<SecondaryMap<ViewId, Rect<f32, f32>>>,
}

impl ViewStorage {
    fn new() -> Self {
        ViewStorage {
            view_ids: Default::default(),
            states: Default::default(),
            handlers: Default::default(),
            child_ids: Default::default(),
            views: Default::default(),
            window_ids: Default::default(),
            parent_view_id: Default::default(),
            focus_index: Default::default(),
            focus_scope: Default::default(),
            pressed: Default::default(),
            visual: Default::default(),
            scroll: Default::default(),
            view_z_index: Default::default(),
            pending_effect_id: Default::default(),
            transform: Default::default(),
            accumulated_transform: Default::default(),
            visual_rect: Default::default(),
        }
    }

    pub fn new_view(&self) -> ViewId {
        let view_id = self.view_ids.lock().insert(());
        self.states
            .write()
            .insert(view_id, RwLock::new(ViewState::new(view_id)));
        self.handlers
            .write()
            .insert(view_id, RwLock::new(ViewHandler::default()));
        view_id
    }

    pub fn add_childs(
        &self,
        parent: ViewId,
        children: impl IntoIterator<Item = Box<dyn View + Send + Sync + 'static>>,
    ) {
        let children: Vec<_> = children.into_iter().collect();
        if children.is_empty() {
            return;
        }

        let children_with_ids: Vec<(ViewId, Box<dyn View + Send + Sync + 'static>)> = children
            .into_iter()
            .map(|child| (child.view_id(), child))
            .collect();

        {
            let mut child_ids_guard = self.child_ids.write();
            let mut parent_view_id_guard = self.parent_view_id.write();

            // 批量更新 parent_view_id
            for (child_id, _) in &children_with_ids {
                parent_view_id_guard.insert(*child_id, parent);
            }

            // 批量更新 child_ids
            if let Some(existing_children) = child_ids_guard.get_mut(parent) {
                existing_children.reserve(children_with_ids.len());
                for (child_id, _) in &children_with_ids {
                    if parent != *child_id {
                        existing_children.push(*child_id);
                    }
                }
            } else {
                let mut new_children = Vec::with_capacity(children_with_ids.len());
                for (child_id, _) in &children_with_ids {
                    if parent != *child_id {
                        new_children.push(*child_id);
                    }
                }
                child_ids_guard.insert(parent, new_children);
            }
        }

        // 批量插入 views
        {
            let mut views_guard = self.views.write();
            for (child_id, child) in children_with_ids {
                views_guard.insert(child_id, RwLock::new(child));
            }
        }

        // 标记父节点 dirty
        if let Some(view_state) = self.states.read().get(parent) {
            view_state.write().dirty_children = true;
        }

        // 通知父节点
        if let Some(view) = self.views.read().get(parent) {
            view.write()
                .on_child_push()
                .error_on_err(format!("on_child_push {{ view_id:{} }}]", parent));
        }
    }

    /// 添加子视图
    pub fn add_child(&self, parent: ViewId, child: Box<dyn View + Send + Sync>) {
        let child_view_id = child.view_id();
        {
            let mut child_ids = self.child_ids.write();

            self.parent_view_id.write().insert(child_view_id, parent);
            // A window is also a view
            if parent != child_view_id {
                if let Some(children) = child_ids.get_mut(parent) {
                    children.push(child_view_id);
                } else {
                    child_ids.insert(parent, vec![child_view_id]);
                }
            }
        }
        self.views.write().insert(child_view_id, RwLock::new(child));
        if let Some(view_state) = VIEW_STORAGE.states.read().get(parent) {
            view_state.write().dirty_children = true;
        }

        // 应该是所有工作完成后再通知，以免出现生命周期问题
        if let Some(view) = self.views.read().get(parent) {
            view.write()
                .on_child_push()
                .error_on_err(format!("on_child_push {{ view_id:{} }}]", parent));
        }
    }

    pub fn reinit_child_view(&self, root_id: ViewId) {
        let Some(window_id) = root_id.window_id() else {
            return;
        };
        let mut child_map = VIEW_STORAGE.child_ids.write();
        let mut window_ids = VIEW_STORAGE.window_ids.write();
        let states = VIEW_STORAGE.states.read();

        let mut stack = Vec::with_capacity(64);
        stack.push(root_id);

        while let Some(view_id) = stack.pop() {
            window_ids.insert(view_id, window_id);

            // 检查是否需要排序
            let needs_sort = if let Some(state) = states.get(view_id) {
                let mut state = state.write();
                if state.dirty_children {
                    state.dirty_children = false;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if let Some(children) = child_map.get_mut(view_id) {
                if needs_sort {
                    children.sort_by(|x, d| x.z_index().cmp(&d.z_index()));
                    // ID 降序？原代码里的 sort_by 是 z_index().cmp()，默认是从小到大。
                    // 但是 add_child 里之前是 sort_by(|x, d| x.z_index().cmp(&d.z_index()))，也是从小到大。
                    // 渲染顺序是从下往上，应该是 z-index 小的先画（在前面），z-index 大的后画（在后面，覆盖）。
                    // 之前的 sort_view_ids_by_z_index 方法是 descending...
                    // 这里我们保持和 add_child 被注释掉的逻辑一致： x.cmp(d) => ascending.
                }

                stack.extend(children.iter().copied()); // 压入栈以便遍历子节点
            }
        }
    }

    pub fn sort_view_ids_by_z_index(&self, childrens: &mut Vec<ViewId>) {
        childrens.sort_by(|a, b| {
            // 1. Z-index 降序：让大的排在前面 (Index 0)
            b.z_index()
                .cmp(&a.z_index())
                // 2. ID 降序：Z-index 相同时，让后创建的（ID大的）排在前面
                // 这样符合“后来居上”的视觉覆盖规则
                .then(b.data().as_ffi().cmp(&a.data().as_ffi()))
        });
    }

    pub fn active_pending_effect_id(&self, view_id: ViewId) {
        if let Some(effect_ids) = self.pending_effect_id.write().remove(view_id) {
            RUNTIME.update_queue.lock().extend(effect_ids);
        }
    }

    pub fn init_window_child(root_id: ViewId, window_id: WindowId) -> Result<(), Error> {
        let child_map = VIEW_STORAGE.child_ids.read();
        let mut window_ids = VIEW_STORAGE.window_ids.write();
        let mut stack = Vec::with_capacity(64);

        stack.push(root_id);
        while let Some(view_id) = stack.pop() {
            window_ids.insert(view_id, window_id);

            if let Some(children) = child_map.get(view_id) {
                stack.extend(children.iter().copied());
            }
        }
        Ok(())
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
            if let Some(view) = self.views.read().get(parent) {
                view.write()
                    .on_child_dispose()
                    .error_on_err(format!("on_child_dispose {{ view_id:{} }}]", parent));
            }
        }

        self.states.write().remove(view_id);
        self.views.write().remove(view_id);
        self.window_ids.write().remove(view_id);
        self.visual_rect.write().remove(view_id);

        // 6. 最后移除 ID
        self.view_ids.lock().remove(view_id);
    }

    pub fn sweep_orphan_views(&self) {
        use rustc_hash::FxHashSet;

        let all_views = self.states.read().keys().collect::<FxHashSet<ViewId>>();

        let mut alive = FxHashSet::default();

        {
            let child_ids = self.child_ids.read();
            let main_roots = WINDOW_ENTRY_MAP
                .iter()
                .map(|v| v.value().view_id)
                .collect::<Vec<ViewId>>();

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
            for root_view_id in main_roots {
                if all_views.contains(&root_view_id) {
                    mark(root_view_id, &child_ids, &mut alive);
                }
            }
        }

        let dead_views = all_views
            .difference(&alive)
            .copied()
            .collect::<Vec<ViewId>>();

        if !dead_views.is_empty() {
            for dead_view_id in dead_views {
                self.dispose_view(dead_view_id);
            }
        }
    }

    // /// 【写操作】重建指定窗口的渲染缓存
    // /// 当布局变化、增删节点、修改 z-index 后调用此方法
    // pub fn rebuild_render_cache(&self, window_id: WindowId) {
    //     // 1. 复用之前的递归排序逻辑生成列表
    //     let sorted_list = self.generate_sorted_list_internal(window_id);
    //
    //     // 2. 获取写锁并更新缓存
    //     // FxHashMap 插入非常快
    //     self.z_index_sort.write().insert(window_id, sorted_list);
    //
    //     // log::trace!("Window {:?} render cache updated.", window_id);
    // }

    // /// 内部私有方法：生成排序列表（逻辑同上一个回答，只是搬到了这里）
    // fn generate_sorted_list_internal(&self, window_id: WindowId) -> Vec<ViewId> {
    //     let Some(root_id) = window_id.try_view_id() else {
    //         return vec![];
    //     };
    //
    //     // 预估容量
    //     let mut list = Vec::with_capacity(128);
    //
    //     // 获取读锁（注意死锁：这里只获取读锁，外面 rebuild_render_cache 获取的是 cache 的写锁，互不冲突）
    //     let child_map = self.child_ids.read();
    //     let state_map = self.states.read();
    //
    //     self.build_recursive(root_id, &child_map, &state_map, &mut list);
    //     list
    // }

    // // 递归构建（同上一个回答）
    // fn build_recursive(
    //     &self,
    //     current_id: ViewId,
    //     child_map: &SecondaryMap<ViewId, Vec<ViewId>>,
    //     state_map: &SecondaryMap<ViewId, RwLock<ViewState>>,
    //     result: &mut Vec<ViewId>,
    // ) {
    //     result.push(current_id); // 先入列（背景）
    //
    //     if let Some(children) = child_map.get(current_id) {
    //         if children.is_empty() {
    //             return;
    //         }
    //
    //         // 提取 (id, z_index, original_index)
    //         let mut sortable: Vec<(ViewId, i32, usize)> = children
    //             .iter()
    //             .enumerate()
    //             .map(|(idx, &id)| {
    //                 let z = state_map.get(id).map(|s| s.read().z_index).unwrap_or(0);
    //                 (id, z, idx)
    //             })
    //             .collect();
    //
    //         // 稳定排序
    //         sortable.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));
    //
    //         // 递归
    //         for (id, _, _) in sortable {
    //             self.build_recursive(id, child_map, state_map, result);
    //         }
    //     }
    // }

    pub fn set_scroll_internal(
        &self,
        view_id: ViewId,
        x: Option<f32>,
        y: Option<f32>,
        is_delta: bool,
    ) {
        // 1. 获取写锁
        // 如果 VIEW_STORAGE.scroll 里没有这个 ID，说明它不是一个可滚动的视图，直接返回
        let mut scroll_map = VIEW_STORAGE.scroll.write();
        let Some(state) = scroll_map.get_mut(view_id) else {
            return;
        };

        let mut changed = false;

        // 2. 处理 X 轴
        if let Some(val_x) = x {
            let target_x = if is_delta {
                state.current.0 + val_x
            } else {
                val_x
            };
            // 钳制：不小于 0.0，不大于 max.0 (layout 算出的 scroll_width)
            let clamped_x = target_x.max(0.0).min(state.max.0);

            if (state.current.0 - clamped_x).abs() > f32::EPSILON {
                state.current.0 = clamped_x;
                changed = true;
            }
        }

        // 3. 处理 Y 轴
        if let Some(val_y) = y {
            let target_y = if is_delta {
                state.current.1 + val_y
            } else {
                val_y
            };
            // 钳制：不小于 0.0，不大于 max.1 (layout 算出的 scroll_height)
            let clamped_y = target_y.max(0.0).min(state.max.1);

            if (state.current.1 - clamped_y).abs() > f32::EPSILON {
                state.current.1 = clamped_y;
                changed = true;
            }
        }

        // 4. 触发更新
        if changed {
            view_id.request_redraw();
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
