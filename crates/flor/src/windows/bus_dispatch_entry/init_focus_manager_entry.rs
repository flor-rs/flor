use crate::error::Error;
use crate::view::{View, VIEW_STORAGE};
use crate::windows::WindowEntryVisit;
use platform::WindowId;

pub fn init_focus_manager_entry(window_id: WindowId) -> Result<(), Error> {
    let root_id = window_id.view_id();
    let mut focus_list = Vec::new();

    // 1. 获取所有需要的锁
    // child_ids: 读锁，用于遍历树结构
    // focus_index & focus_scope: 写锁，用于"取走"数据 (remove)
    // 只要锁的获取顺序一致，或者同时获取，就不会死锁
    let child_ids_guard = VIEW_STORAGE.child_ids.read();
    let views_guard = VIEW_STORAGE.views.read();
    let mut focus_index_guard = VIEW_STORAGE.focus_index.write();
    let mut focus_scope_guard = VIEW_STORAGE.focus_scope.write();

    // 2. 准备遍历栈 (DFS)
    // 栈元素: (ViewId, 父级传递下来的累加 Scope 基数)
    let mut stack = vec![(root_id, 0u32)];

    while let Some((current_id, parent_offset)) = stack.pop() {
        // A. 处理 Scope (核心逻辑)
        // 尝试从 storage 中 remove 掉当前节点的 scope 配置
        // 如果有，则累加到 parent_offset 上，作为自己和子节点的新基数
        // 如果没有，scope_val 就是 0
        let scope_val = focus_scope_guard.remove(current_id).unwrap_or(0);
        let current_total_offset = parent_offset + scope_val;

        // B. 处理 Focus Index
        // 尝试从 storage 中 remove 掉当前节点的 index 配置
        // 最终索引 = 当前累加基数 + 局部索引
        if let Some(local_index) = focus_index_guard.remove(current_id) {
            let final_index = current_total_offset + local_index;

            // 查询控件的虚拟焦点数量，展开为多个条目
            let count = views_guard
                .get(current_id)
                .map(|v| v.read().on_focus_count())
                .unwrap_or(1)
                .max(1);

            for vi in 0..count {
                focus_list.push((final_index, current_id, vi));
            }
        }

        // C. 将子节点压栈
        if let Some(children) = child_ids_guard.get(current_id) {
            // 使用 rev() 倒序压栈，这样 pop 出来时是从第一个子节点开始处理
            // 将计算好的 current_total_offset 继续传递下去
            for child_id in children.iter().rev() {
                stack.push((*child_id, current_total_offset));
            }
        }
    }

    // 3. 提交给 FocusManager
    // 此时 focus_list 里的数据已经是计算好 Scope 偏移的最终结果了
    window_id
        .entry_mut()
        .map(|mut v| v.focus_manager.set_focus_list(focus_list));

    Ok(())
}
