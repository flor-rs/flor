use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use rustc_hash::FxHashMap;

/// 焦点作用域条目（用于 Modal 等场景）
#[derive(Debug, Clone)]
pub struct FocusScopeEntry {
    /// Modal 的根控件
    pub root_view_id: ViewId,
    /// 打开 Modal 前的焦点位置
    pub previous_focus: Option<ViewId>,
}

#[derive(Default, Debug)]
pub struct FocusManager {
    focus_list: Vec<(u32, ViewId)>, // 按顺序存储
    // view_id , focus_list index
    view_index_map: FxHashMap<ViewId, usize>, // ViewId -> Vec 索引
    current: Option<usize>,

    /// 焦点作用域栈（用于 Modal 等场景）
    /// 栈顶的 scope 限制 Tab 键只在该 scope 的子树内循环
    scope_stack: Vec<FocusScopeEntry>,
}

impl FocusManager {
    pub fn set_focus_list(&mut self, focus_list: Vec<(u32, ViewId)>) {
        self.focus_list = focus_list;
        if self.focus_list.len() > 0 {
            self.focus_list.sort_by_key(|(idx, vid)| (*idx, *vid));
            // 重新索引
            self.view_index_map = self
                .focus_list
                .iter()
                .enumerate()
                .map(|(idx, (_, vid))| (*vid, idx))
                .collect();
        }
    }

    /// 更新或插入焦点顺序
    pub fn update_focused(&mut self, view_id: ViewId, focus_index: u32) {
        // 记录旧的view_id
        let old_view_id = self.index_to_view_id(self.current);

        // 清理旧的要操作的节点数据
        if let Some(i) = self.view_index_map.remove(&view_id) {
            self.focus_list.remove(i);
        }

        // 为0是禁用焦点功能，所以不为0才插入
        if focus_index > 0 {
            {
                // 插入新的
                self.focus_list.push((focus_index, view_id));
                self.focus_list.sort_by_key(|(idx, vid)| (*idx, *vid));
            }
        }
        // 重新索引
        self.view_index_map = self
            .focus_list
            .iter()
            .enumerate()
            .map(|(idx, (_, vid))| (*vid, idx))
            .collect();

        // 恢复之前的焦点
        if let Some(old_view_id) = old_view_id {
            self.current = self.view_index_map.get(&old_view_id).copied();
        }

        let new_view_id = self.index_to_view_id(self.current);
        self.switch_focus(old_view_id, new_view_id);
    }

    pub fn next(&mut self) {
        let scoped_list = self.get_scoped_focus_list();
        let len = scoped_list.len();
        if len == 0 {
            return;
        }

        let old_view_id = self.index_to_view_id(self.current);

        // 在作用域列表中找到当前位置
        let current_pos_in_scope = old_view_id
            .and_then(|vid| scoped_list.iter().position(|(_, v)| *v == vid));

        // 计算下一个位置
        let next_pos = match current_pos_in_scope {
            None => 0,
            Some(pos) => (pos + 1) % len,
        };

        // 获取新的 ViewId 并更新 current
        let new_view_id = scoped_list.get(next_pos).map(|(_, vid)| *vid);
        if let Some(vid) = new_view_id {
            self.current = self.view_index_map.get(&vid).copied();
        }

        self.switch_focus(old_view_id, new_view_id);
    }

    pub fn prev(&mut self) {
        let scoped_list = self.get_scoped_focus_list();
        let len = scoped_list.len();
        if len == 0 {
            return;
        }

        let old_view_id = self.index_to_view_id(self.current);

        // 在作用域列表中找到当前位置
        let current_pos_in_scope = old_view_id
            .and_then(|vid| scoped_list.iter().position(|(_, v)| *v == vid));

        // 计算上一个位置
        let prev_pos = match current_pos_in_scope {
            None => 0,
            Some(pos) => {
                if pos == 0 {
                    len - 1
                } else {
                    pos - 1
                }
            }
        };

        // 获取新的 ViewId 并更新 current
        let new_view_id = scoped_list.get(prev_pos).map(|(_, vid)| *vid);
        if let Some(vid) = new_view_id {
            self.current = self.view_index_map.get(&vid).copied();
        }

        self.switch_focus(old_view_id, new_view_id);
    }

    /// 获取当前作用域内的焦点列表
    fn get_scoped_focus_list(&self) -> Vec<(u32, ViewId)> {
        match self.scope_stack.last() {
            None => self.focus_list.clone(),
            Some(scope) => self
                .focus_list
                .iter()
                .filter(|(_, vid)| Self::is_descendant_of(*vid, scope.root_view_id))
                .cloned()
                .collect(),
        }
    }

    /// 检查 view_id 是否是 root_id 的后代（或自身）
    fn is_descendant_of(view_id: ViewId, root_id: ViewId) -> bool {
        if view_id == root_id {
            return true;
        }

        let parent_map = VIEW_STORAGE.parent_view_id.read();
        let mut current = view_id;

        // 向上遍历父节点
        while let Some(parent) = parent_map.get(current).copied() {
            if parent == root_id {
                return true;
            }
            current = parent;
        }

        false
    }

    // ========================================================================
    // 焦点作用域 API
    // ========================================================================

    /// 推入一个焦点作用域
    ///
    /// 调用后，Tab 键只在 `root_view_id` 的子树内循环。
    /// 同时记录当前焦点位置，以便 pop 时恢复。
    ///
    /// 适用场景：Modal Dialog、Popup、侧边栏等需要限制焦点范围的场景。
    pub fn push_focus_scope(&mut self, root_view_id: ViewId) {
        let previous_focus = self.current_view_id();

        self.scope_stack.push(FocusScopeEntry {
            root_view_id,
            previous_focus,
        });

        // 如果当前焦点不在新作用域内，尝试聚焦到作用域内的第一个控件
        if let Some(current) = self.current_view_id() {
            if !Self::is_descendant_of(current, root_view_id) {
                self.focus_first_in_scope();
            }
        } else {
            self.focus_first_in_scope();
        }
    }

    /// 弹出当前焦点作用域
    ///
    /// 恢复到之前的焦点位置。
    pub fn pop_focus_scope(&mut self) {
        if let Some(scope) = self.scope_stack.pop() {
            // 恢复之前的焦点
            if let Some(previous) = scope.previous_focus {
                self.set_focus(previous);
            }
        }
    }

    /// 获取当前作用域的根控件（如果有）
    pub fn current_scope_root(&self) -> Option<ViewId> {
        self.scope_stack.last().map(|s| s.root_view_id)
    }

    /// 聚焦到当前作用域内的第一个控件
    fn focus_first_in_scope(&mut self) {
        let scoped_list = self.get_scoped_focus_list();
        if let Some((_, first_vid)) = scoped_list.first() {
            self.set_focus(*first_vid);
        }
    }
    pub fn is_focused(&self, view_id: ViewId) -> bool {
        self.view_index_map.get(&view_id) == self.current.as_ref()
    }

    pub fn current_view_id(&self) -> Option<ViewId> {
        self.index_to_view_id(self.current)
    }

    fn index_to_view_id(&self, index: Option<usize>) -> Option<ViewId> {
        index
            .and_then(|index| self.focus_list.get(index))
            .map(|(_, view_id)| *view_id)
    }

    fn switch_focus(&self, old_view_id: Option<ViewId>, new_view_id: Option<ViewId>) {
        if old_view_id == new_view_id {
            return;
        }
        if let Some(view_id) = old_view_id {
            view_id.call_focus_lost();
        }
        if let Some(view_id) = new_view_id {
            view_id.call_focus_gained();
        }
    }

    pub fn set_focus(&mut self, view_id: ViewId) {
        let new_index = match self.view_index_map.get(&view_id).copied() {
            Some(idx) => idx,
            None => return,
        };
        let old_view_id = self.index_to_view_id(self.current);
        let new_view_id = self.index_to_view_id(Some(new_index));
        if old_view_id == new_view_id {
            return;
        }
        self.current = Some(new_index);

        if new_view_id.is_some() {
            self.switch_focus(old_view_id, new_view_id);
        }
    }
}
