use crate::error::Error;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use platform::WindowId;

/// 扁平化的 bus_create 入口实现
/// 使用显式栈替代递归，提升性能
pub fn create_entry(window_id: WindowId) -> Result<(), Error> {
    let root_id = window_id.view_id();
    let child_map = VIEW_STORAGE.child_ids.read();
    let views = VIEW_STORAGE.views.read();

    // 使用栈进行深度优先遍历
    let mut stack = Vec::with_capacity(64);
    stack.push(root_id);

    while let Some(view_id) = stack.pop() {
        // 1. 调用当前节点的 call_create
        if let Some(view) = views.get(view_id) {
            view.write().call_create()?;
        }

        // 2. 将子节点压入栈（逆序以保持遍历顺序）
        if let Some(children) = child_map.get(view_id) {
            for child_id in children.iter().rev() {
                stack.push(*child_id);
            }
        }
    }

    Ok(())
}
