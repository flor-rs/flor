use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use rustc_hash::FxHashMap;

/// 焦点作用域条目（用于 Modal 等场景）
#[derive(Debug, Clone)]
pub struct FocusScopeEntry {
    /// Modal 的根控件
    pub root_view_id: ViewId,
    /// 打开 Modal 前的焦点位置
    pub previous_focus: Option<(ViewId, u16)>,
}

#[derive(Default, Debug)]
pub struct FocusManager {
    /// (排序键, ViewId, 虚拟焦点序号)，按排序键排序
    focus_list: Vec<(u32, ViewId, u16)>,
    /// (ViewId, 虚拟焦点序号) -> focus_list 索引，用于精确定位
    index_map: FxHashMap<(ViewId, u16), usize>,
    current: Option<usize>,

    /// 焦点作用域栈（用于 Modal 等场景）
    /// 栈顶的 scope 限制 Tab 键只在该 scope 的子树内循环
    scope_stack: Vec<FocusScopeEntry>,
}

impl FocusManager {
    pub fn set_focus_list(&mut self, focus_list: Vec<(u32, ViewId, u16)>) {
        self.focus_list = focus_list;
        if self.focus_list.len() > 0 {
            self.focus_list
                .sort_by_key(|(idx, vid, vi)| (*idx, *vid, *vi));
            self.rebuild_index_map();
        }
    }

    /// 更新或插入焦点顺序（运行时动态更新，按单焦点处理）
    pub fn update_focused(&mut self, view_id: ViewId, focus_index: u32) {
        let old_entry = self.current_entry();

        // 清理该 ViewId 的所有旧条目
        self.focus_list.retain(|(_, vid, _)| *vid != view_id);

        // 为0是禁用焦点功能，所以不为0才插入
        if focus_index > 0 {
            self.focus_list.push((focus_index, view_id, 0));
            self.focus_list
                .sort_by_key(|(idx, vid, vi)| (*idx, *vid, *vi));
        }

        self.rebuild_index_map();

        // 恢复之前的焦点
        if let Some((old_vid, old_vi)) = old_entry {
            self.current = self.index_map.get(&(old_vid, old_vi)).copied();
        }

        let new_entry = self.current_entry();
        match (old_entry, new_entry) {
            (Some(old), Some(new)) => self.switch_focus(old, new),
            (Some((vid, vi)), None) => vid.call_blur(vi),
            (None, Some((vid, vi))) => vid.call_focus(vi),
            (None, None) => {}
        }
    }

    pub fn next(&mut self) {
        let scoped_list = self.get_scoped_focus_list();
        let len = scoped_list.len();
        if len == 0 {
            return;
        }

        let old_entry = self.current_entry();

        // 在作用域列表中找到当前位置
        let current_pos_in_scope = old_entry.and_then(|(vid, vi)| {
            scoped_list
                .iter()
                .position(|(_, v, i)| *v == vid && *i == vi)
        });

        // 计算下一个位置
        let next_pos = match current_pos_in_scope {
            None => 0,
            Some(pos) => (pos + 1) % len,
        };

        // new 必定存在（len > 0 且 next_pos 是合法索引）
        let (_, new_vid, new_vi) = scoped_list[next_pos];
        let new = (new_vid, new_vi);
        self.current = self.index_map.get(&new).copied();

        match old_entry {
            Some(old) => self.switch_focus(old, new),
            None => new_vid.call_focus(new_vi),
        }
    }

    pub fn prev(&mut self) {
        let scoped_list = self.get_scoped_focus_list();
        let len = scoped_list.len();
        if len == 0 {
            return;
        }

        let old_entry = self.current_entry();

        // 在作用域列表中找到当前位置
        let current_pos_in_scope = old_entry.and_then(|(vid, vi)| {
            scoped_list
                .iter()
                .position(|(_, v, i)| *v == vid && *i == vi)
        });

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

        // new 必定存在（len > 0 且 prev_pos 是合法索引）
        let (_, new_vid, new_vi) = scoped_list[prev_pos];
        let new = (new_vid, new_vi);
        self.current = self.index_map.get(&new).copied();

        match old_entry {
            Some(old) => self.switch_focus(old, new),
            None => new_vid.call_focus(new_vi),
        }
    }

    /// 获取当前作用域内的焦点列表
    fn get_scoped_focus_list(&self) -> Vec<(u32, ViewId, u16)> {
        match self.scope_stack.last() {
            None => self.focus_list.clone(),
            Some(scope) => self
                .focus_list
                .iter()
                .filter(|(_, vid, _)| Self::is_descendant_of(*vid, scope.root_view_id))
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
        let previous_focus = self.current_entry();

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
            if let Some((vid, vi)) = scope.previous_focus {
                self.set_focus(vid, vi);
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
        if let Some((_, vid, vi)) = scoped_list.first() {
            self.set_focus(*vid, *vi);
        }
    }

    /// 宽松判定：该 ViewId 上是否持有焦点（不区分虚拟焦点序号）
    pub fn is_focused(&self, view_id: ViewId) -> bool {
        self.current_view_id() == Some(view_id)
    }

    /// 获取当前焦点所在的 ViewId
    pub fn current_view_id(&self) -> Option<ViewId> {
        self.current_entry().map(|(vid, _)| vid)
    }

    /// 获取当前焦点的完整条目 (ViewId, 虚拟焦点序号)
    fn current_entry(&self) -> Option<(ViewId, u16)> {
        self.current
            .and_then(|index| self.focus_list.get(index))
            .map(|(_, vid, vi)| (*vid, *vi))
    }

    fn switch_focus(&self, old: (ViewId, u16), new: (ViewId, u16)) {
        if old == new {
            return;
        }
        old.0.call_blur(old.1);
        new.0.call_focus(new.1);
    }

    pub fn set_focus(&mut self, view_id: ViewId, virtual_index: u16) {
        let new_index = match self.index_map.get(&(view_id, virtual_index)).copied() {
            Some(idx) => idx,
            None => return,
        };

        let old_entry = self.current_entry();
        let new_entry = self
            .focus_list
            .get(new_index)
            .map(|(_, vid, vi)| (*vid, *vi));

        if old_entry == new_entry {
            return;
        }
        self.current = Some(new_index);
        let new_entry = new_entry.unwrap(); // new_index 来自 index_map，必定有值
        match old_entry {
            Some(old) => self.switch_focus(old, new_entry),
            None => new_entry.0.call_focus(new_entry.1),
        }
    }

    /// 重建索引映射
    #[inline]
    fn rebuild_index_map(&mut self) {
        self.index_map = self
            .focus_list
            .iter()
            .enumerate()
            .map(|(idx, (_, vid, vi))| ((*vid, *vi), idx))
            .collect();
    }
}
