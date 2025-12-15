use crate::view::view_id::ViewId;
use rustc_hash::FxHashMap;

#[derive(Default, Debug)]
pub struct FocusManager {
    focus_list: Vec<(u32, ViewId)>, // 按顺序存储
    // view_id , focus_list index
    view_index_map: FxHashMap<ViewId, usize>, // ViewId -> Vec 索引
    current: Option<usize>,
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
        let len = self.focus_list.len();
        if len == 0 || len == 1 {
            return;
        }

        let old_view_id = self.index_to_view_id(self.current);
        self.current = Some(match self.current {
            None => 0,
            Some(v) => (v + 1) % len, // 循环
        });
        let new_view_id = self.index_to_view_id(self.current);
        self.switch_focus(old_view_id, new_view_id);
    }

    pub fn prev(&mut self) {
        let len = self.focus_list.len();
        if len == 0 || len == 1 {
            return;
        }

        let old_view_id = self.index_to_view_id(self.current);
        self.current = Some(match self.current {
            None => 0,
            Some(v) => {
                if v == 0 {
                    len - 1
                } else {
                    v - 1
                }
            }
        });
        let new_view_id = self.index_to_view_id(self.current);
        self.switch_focus(old_view_id, new_view_id);
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
            view_id.on_focus_lost();
        }
        if let Some(view_id) = new_view_id {
            view_id.on_focus_gained();
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
