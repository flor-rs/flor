use crate::error::Error;
use crate::view::collect_layout_children;
use crate::view::state_selector::CalcTaffyStyle;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus::render_from_view_id;
use crate::windows::entry::WindowEntryVisit;
use flor_base::platform::WindowApi;
use flor_base::types::Transform2D;
use log::{trace, warn};
use platform::WindowId;
use std::ops::DerefMut;
use taffy::{AvailableSpace, Size, Style};

pub fn refresh_layout_entry(window_id: WindowId) -> Result<(), Error> {
    trace!("enter relayout");

    let Some(window_entry) = window_id.entry() else {
        warn!("window not found entry in re_layout_entry function.");
        return Ok(());
    };

    let layout_tree = &mut window_entry.taffy_tree.write();
    let view_id = window_entry.view_id;

    let states = VIEW_STORAGE.states.read();
    let Some(view_state_cell) = states.get(view_id) else {
        warn!("View storage's states not found view_id:{view_id:?}");
        return Ok(());
    };

    let view_state = view_state_cell.read();
    let old_node_id = view_state.node_id;

    let mut style_update = view_state
        .layout_style
        .calc_update_taffy_style(view_id.control_state());

    // 这里的特殊逻辑：如果 style 有更新，必须强制加上 100% 的尺寸限制
    if let Some(s) = &mut style_update {
        s.size = Size::from_percent(1.0, 1.0);
    }

    drop(view_state);

    let children = collect_layout_children(view_id, layout_tree)?;

    let root_node_id = match (old_node_id, style_update) {
        (Some(node_id), None) => {
            if !children.is_empty() {
                layout_tree.set_children(node_id, &children)?;
            }
            node_id
        }
        (Some(node_id), Some(new_style)) => {
            layout_tree.set_style(node_id, new_style)?;
            if !children.is_empty() {
                layout_tree.set_children(node_id, &children)?;
            }
            node_id
        }
        (None, style_opt) => {
            let style = style_opt.unwrap_or_else(|| Style {
                size: Size::from_percent(1.0, 1.0),
                ..Default::default()
            });
            if children.is_empty() {
                layout_tree.new_leaf_with_context(style, view_id)?
            } else {
                layout_tree.new_with_children(style, &children)?
            }
        }
    };

    if old_node_id != Some(root_node_id) {
        let mut view_state = view_state_cell.write();
        view_state.node_id = Some(root_node_id);
    }

    let client_size = window_id.get_client_size()?;
    layout_tree.compute_layout_with_measure(
        root_node_id,
        Size {
            height: AvailableSpace::Definite(client_size.1 as f32),
            width: AvailableSpace::Definite(client_size.0 as f32),
        },
        |known_dimensions, available_space, _node_id, node_context_view_id, style| {
            if let Some(view_id) = node_context_view_id {
                if let Some(dyn_view) = VIEW_STORAGE.views.read().get(*view_id) {
                    if let Some(render) = render_from_view_id(*view_id).as_deref() {
                        let mut render = render.write();
                        let render = render.deref_mut();
                        let mut view = dyn_view.write();
                        return view
                            .on_measure(known_dimensions, available_space, style, render)
                            .unwrap_or(Size::ZERO);
                    }
                }
            }
            Size::ZERO
        },
    )?;

    {
        trace!("bus_update_layout begin");
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write().bus_update_layout(layout_tree, (0.0, 0.0))?;
        }
        trace!("bus_update_layout end");
    }

    // 计算累积变换：从根控件遍历，累加 transform 到 accumulated_transform
    compute_accumulated_transforms(view_id);

    Ok(())
}

/// 计算累积变换矩阵
///
/// accumulated_transform 代表：控件局部坐标(0,0) → 窗口坐标 的完整变换
///
/// 变换链（按行向量乘法顺序）：
/// accumulated = parent_accumulated 
///     * translation(layout.location.x, layout.location.y)  // 平移到当前控件位置
///     * local_transform                                     // 控件自身变换
///     * translation(-scroll_x, -scroll_y)                   // scroll 偏移
fn compute_accumulated_transforms(start_view_id: ViewId) {
    let child_ids = VIEW_STORAGE.child_ids.read();
    let states = VIEW_STORAGE.states.read();
    let transform_map = VIEW_STORAGE.transform.read();
    let scroll_map = VIEW_STORAGE.scroll.read();
    let mut accumulated_map = VIEW_STORAGE.accumulated_transform.write();

    // (view_id, parent_accumulated_transform)
    // 根节点的父变换是 Identity（或 None）
    let mut stack = vec![(start_view_id, Transform2D::IDENTITY)];

    while let Some((view_id, parent_accumulated)) = stack.pop() {
        // 1. 获取当前控件的 layout.location（相对于父控件的位置）
        let location = states
            .get(view_id)
            .map(|s| {
                let state = s.read();
                (state.layout.location.x, state.layout.location.y)
            })
            .unwrap_or((0.0, 0.0));

        // 2. 获取控件自身的 transform
        let local_transform = transform_map.get(view_id).copied();

        // 3. 获取 scroll offset
        let scroll_offset = scroll_map
            .get(view_id)
            .map(|s| s.current)
            .unwrap_or((0.0, 0.0));

        // 4. 计算当前控件的累积变换
        // 顺序：parent_accumulated * translation(location) * local_transform * translation(-scroll)
        let mut current_accumulated = parent_accumulated
            .then_translate(location.0, location.1);

        if let Some(local_tf) = local_transform {
            current_accumulated = current_accumulated * local_tf;
        }

        // scroll 只影响子控件，所以要作为传递给子控件的变换的一部分
        // 但对于当前控件自身的命中测试，不需要 scroll
        // 所以这里我们存储不含 scroll 的版本，给子控件传递含 scroll 的版本

        // 存储当前控件的累积变换（用于命中测试自身）
        accumulated_map.insert(view_id, current_accumulated);

        // 5. 计算传递给子控件的累积变换（包含 scroll）
        let child_parent_accumulated = if scroll_offset.0 != 0.0 || scroll_offset.1 != 0.0 {
            current_accumulated.then_translate(-scroll_offset.0, -scroll_offset.1)
        } else {
            current_accumulated
        };

        // 6. 处理子控件
        if let Some(children) = child_ids.get(view_id) {
            for &child_id in children {
                stack.push((child_id, child_parent_accumulated));
            }
        }
    }
}
