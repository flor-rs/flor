use crate::error::Error;
use crate::view::ControlState;
use crate::view::ViewId;
use crate::view::ViewState;
use crate::view::VIEW_STORAGE;
use crate::windows::bus::render;
use crate::windows::WindowEntryVisit;
use flor_base::platform::WindowApi;
use flor_base::types::{Rect, Transform2D};
use log::{debug, trace, warn};
use parking_lot::RwLock;
use platform::WindowId;
use slotmap::SecondaryMap;
use std::ops::{Deref, DerefMut};
use std::time::Instant;
use taffy::{AvailableSpace, NodeId, Size, Style, TaffyTree, TraversePartialTree};

/// 检查 taffy 节点的 children 是否与新的 children 相同
#[inline]
fn children_changed(taffy: &TaffyTree<ViewId>, node_id: NodeId, new_children: &[NodeId]) -> bool {
    let old_children = taffy.child_ids(node_id);
    if taffy.child_count(node_id) != new_children.len() {
        return true;
    }
    for (old, new) in old_children.zip(new_children.iter()) {
        if old != *new {
            return true;
        }
    }
    false
}

pub fn refresh_layout_entry(window_id: WindowId) -> Result<(), Error> {
    let start_time = Instant::now();
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

    let style_update = view_state
        .layout_style
        .get_data_if_changed(view_id.control_state());

    drop(view_state);

    let instant = Instant::now();

    let children = collect_layout_children(view_id, states.deref(), layout_tree)?;

    debug!("collect_layout_children: {:?}", instant.elapsed());

    let root_node_id = match (old_node_id, style_update) {
        (Some(node_id), None) => {
            if !children.is_empty() && children_changed(layout_tree, node_id, &children) {
                layout_tree.set_children(node_id, &children)?;
            }
            node_id
        }
        (Some(node_id), Some(new_style)) => {
            layout_tree.set_style(node_id, new_style)?;
            if !children.is_empty() && children_changed(layout_tree, node_id, &children) {
                layout_tree.set_children(node_id, &children)?;
            }
            node_id
        }
        (None, style_opt) => {
            let style = style_opt.unwrap_or_else(|| Style {
                size: Size::auto(),
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
    let mut measure_call_count = 0u32;
    let compute_layout_start = Instant::now();

    let Some(render) = render(window_id) else {
        return Ok(());
    };
    let mut render = render.write();
    let render = render.deref_mut();
    let views = VIEW_STORAGE.views.read();

    // 获取计算 control_state 所需的读锁
    let measure_states = VIEW_STORAGE.states.read();
    let pressed = VIEW_STORAGE.pressed.read();

    layout_tree.compute_layout_with_measure(
        root_node_id,
        Size {
            height: AvailableSpace::Definite(client_size.1 as f32),
            width: AvailableSpace::Definite(client_size.0 as f32),
        },
        |known_dimensions, available_space, _node_id, node_context_view_id, style| {
            measure_call_count += 1;
            if let Some(view_id) = node_context_view_id {
                if let Some(dyn_view) = views.get(*view_id) {
                    // 计算 control_state（按优先级：Disabled > Active > Focus > Hover > Normal）
                    let control_state =
                        if let Some(view_state) = measure_states.get(*view_id).map(|s| s.read()) {
                            match true {
                                _ if view_state.disable => ControlState::Disabled,
                                _ if pressed.get(*view_id).is_some() => ControlState::Active,
                                _ if window_entry.focus_manager.is_focused(*view_id) => {
                                    ControlState::Focus
                                }
                                _ if window_entry.hover_id == Some(*view_id) => ControlState::Hover,
                                _ => ControlState::Normal,
                            }
                        } else {
                            ControlState::Normal
                        };

                    let mut view = dyn_view.write();
                    return view
                        .on_measure(
                            known_dimensions,
                            available_space,
                            style,
                            control_state,
                            render,
                        )
                        .unwrap_or(Size::ZERO);
                }
            }
            Size::ZERO
        },
    )?;
    let compute_layout_elapsed = compute_layout_start.elapsed();
    debug!(
        "compute_layout_with_measure completed: elapsed={:?}, measure_call_count={}",
        compute_layout_elapsed, measure_call_count
    );

    bus_update_layout_iterative(view_id, layout_tree, (0.0, 0.0))?;

    // 计算累积变换：从根控件遍历，累加 transform 到 accumulated_transform
    compute_accumulated_transforms(view_id, states.deref());

    // 计算并缓存所有视图的 visual_rect，用于绘制时的快速剔除
    compute_visual_rects(view_id);

    let total_elapsed = start_time.elapsed();
    trace!(
        "refresh_layout_entry completed: total_elapsed={:?}",
        total_elapsed
    );
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
fn compute_accumulated_transforms(
    start_view_id: ViewId,
    states: &SecondaryMap<ViewId, RwLock<ViewState>>,
) {
    let child_ids = VIEW_STORAGE.child_ids.read();
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
        let mut current_accumulated = parent_accumulated.then_translate(location.0, location.1);

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

enum LayoutStage {
    /// 进入节点：准备处理节点及其子节点
    Enter,
    /// 退出节点：子节点已处理完成，收集结果并构建当前节点
    Exit {
        old_node_id: Option<NodeId>,
        style: Option<Style>,
        /// 子节点的 NodeId 数量
        children_count: usize,
    },
}

struct LayoutFrame {
    view_id: ViewId,
    stage: LayoutStage,
}

pub fn collect_layout_children(
    parent_id: ViewId,
    states: &SecondaryMap<ViewId, RwLock<ViewState>>,
    taffy: &mut TaffyTree<ViewId>,
) -> Result<Vec<NodeId>, Error> {
    // 一次性读取所有需要的全局锁，避免循环中重复加锁
    let child_ids_map = VIEW_STORAGE.child_ids.read();
    let pressed = VIEW_STORAGE.pressed.read();

    // 获取 window_entry（用于 focus 和 hover 状态）
    let Some(window_id) = parent_id.window_id() else {
        return Ok(Vec::new());
    };
    let Some(window_entry) = window_id.entry() else {
        return Ok(Vec::new());
    };

    // 读取顶层的子节点列表
    let Some(top_children) = child_ids_map.get(parent_id).cloned() else {
        return Ok(Vec::new());
    };

    let mut result = Vec::with_capacity(top_children.len());

    // 使用栈来处理每个顶层子节点及其后代
    for top_child_id in top_children {
        let mut stack = vec![LayoutFrame {
            view_id: top_child_id,
            stage: LayoutStage::Enter,
        }];

        // 存储已处理完成的节点结果
        let mut node_results = Vec::new();

        while let Some(frame) = stack.pop() {
            match frame.stage {
                LayoutStage::Enter => {
                    let view_id = frame.view_id;

                    let Some(view_state_cell) = states.get(view_id) else {
                        return Err(Error::ControlUnregistered(view_id));
                    };

                    let view_state = view_state_cell.read();
                    let old_node_id = view_state.node_id;

                    // 计算 control_state（按优先级：Disabled > Active > Focus > Hover > Normal）
                    let control_state = match true {
                        _ if view_state.disable => ControlState::Disabled,
                        _ if pressed.get(view_id).is_some() => ControlState::Active,
                        _ if window_entry.focus_manager.is_focused(view_id) => ControlState::Focus,
                        _ if window_entry.hover_id == Some(view_id) => ControlState::Hover,
                        _ => ControlState::Normal,
                    };

                    let style = view_state.layout_style.get_data_if_changed(control_state);
                    drop(view_state);

                    // 获取当前节点的子节点
                    let children_ids = child_ids_map.get(view_id).cloned().unwrap_or_default();

                    if children_ids.is_empty() {
                        // 无子节点，直接处理
                        let node_id = process_leaf_node(view_id, old_node_id, style, taffy)?;
                        node_results.push(node_id);
                    } else {
                        // 有子节点，压栈退出阶段（用于收集子节点结果）
                        stack.push(LayoutFrame {
                            view_id,
                            stage: LayoutStage::Exit {
                                old_node_id,
                                style,
                                children_count: children_ids.len(),
                            },
                        });

                        // 逆序压栈子节点（保持遍历顺序）
                        for &child_id in children_ids.iter().rev() {
                            stack.push(LayoutFrame {
                                view_id: child_id,
                                stage: LayoutStage::Enter,
                            });
                        }
                    }
                }

                LayoutStage::Exit {
                    old_node_id,
                    style,
                    children_count,
                } => {
                    // 收集子节点结果
                    let children_start = node_results.len() - children_count;
                    let children = node_results.drain(children_start..).collect::<Vec<_>>();

                    // 处理当前节点
                    let node_id =
                        process_parent_node(frame.view_id, old_node_id, style, &children, taffy)?;

                    node_results.push(node_id);
                }
            }
        }

        // 此时 node_results 应该只有一个元素：顶层子节点的 NodeId
        if let Some(node_id) = node_results.into_iter().next() {
            result.push(node_id);
        }
    }

    Ok(result)
}

/// 处理叶子节点（无子节点）
#[inline]
fn process_leaf_node(
    view_id: ViewId,
    old_node_id: Option<NodeId>,
    style: Option<Style>,
    taffy: &mut TaffyTree<ViewId>,
) -> Result<NodeId, Error> {
    let node_id = match (old_node_id, style) {
        (Some(node_id), None) => node_id,
        (Some(node_id), Some(new_style)) => {
            taffy.set_style(node_id, new_style)?;
            node_id
        }
        (None, Some(style)) => taffy.new_leaf_with_context(style, view_id)?,
        // SAFETY: 首次布局时必定有 style（calc_update_taffy_style 返回 Some）
        // 此分支在正常逻辑下不可达
        (None, None) => unreachable!(),
    };

    // 更新 view_state 中的 node_id
    if let Some(view_state_cell) = VIEW_STORAGE.states.read().get(view_id) {
        view_state_cell.write().node_id = Some(node_id);
    }

    Ok(node_id)
}

/// 处理父节点（有子节点）
#[inline]
fn process_parent_node(
    view_id: ViewId,
    old_node_id: Option<NodeId>,
    style: Option<Style>,
    children: &[NodeId],
    taffy: &mut TaffyTree<ViewId>,
) -> Result<NodeId, Error> {
    let node_id = match (old_node_id, style) {
        (Some(node_id), None) => {
            if !children.is_empty() && children_changed(taffy, node_id, children) {
                taffy.set_children(node_id, children)?;
            }
            node_id
        }
        (Some(node_id), Some(new_style)) => {
            taffy.set_style(node_id, new_style)?;
            if !children.is_empty() && children_changed(taffy, node_id, children) {
                taffy.set_children(node_id, children)?;
            }
            node_id
        }
        (None, Some(style)) => {
            if children.is_empty() {
                taffy.new_leaf_with_context(style, view_id)?
            } else {
                taffy.new_with_children(style, children)?
            }
        }
        // SAFETY: 首次布局时必定有 style（calc_update_taffy_style 返回 Some）
        // 此分支在正常逻辑下不可达
        (None, None) => unreachable!(),
    };

    // 更新 view_state 中的 node_id
    if let Some(view_state_cell) = VIEW_STORAGE.states.read().get(view_id) {
        view_state_cell.write().node_id = Some(node_id);
    }

    Ok(node_id)
}

/// 非递归更新布局：遍历视图树，更新每个节点的 layout 和 abs_location
pub fn bus_update_layout_iterative(
    root_view_id: ViewId,
    taffy: &mut TaffyTree<ViewId>,
    initial_abs_location: (f32, f32),
) -> Result<(), Error> {
    let child_ids_map = VIEW_STORAGE.child_ids.read();
    let states = VIEW_STORAGE.states.read();
    let mut scroll_map = VIEW_STORAGE.scroll.write();

    let mut stack = vec![(root_view_id, initial_abs_location)];

    while let Some((view_id, parent_abs_location)) = stack.pop() {
        // 更新当前节点的 layout 和 abs_location
        let current_abs_location = if let Some(state_cell) = states.get(view_id) {
            let mut state = state_cell.write();

            if let Some(node_id) = state.node_id {
                state.layout = *taffy.layout(node_id)?;

                // 如果是可滚动视图，更新 scroll.max
                let scroll_width = state.layout.scroll_width();
                let scroll_height = state.layout.scroll_height();
                if scroll_width > 0.0 || scroll_height > 0.0 {
                    if let Some(scroll_state) = scroll_map.get_mut(view_id) {
                        scroll_state.max = (scroll_width, scroll_height);
                    }
                }
            }

            // 计算绝对位置 = 父级绝对位置 + 自身相对位置
            let abs_location = (
                parent_abs_location.0 + state.layout.location.x,
                parent_abs_location.1 + state.layout.location.y,
            );
            state.abs_location = abs_location;

            abs_location
        } else {
            parent_abs_location
        };

        // 将子节点压栈
        if let Some(children) = child_ids_map.get(view_id) {
            for &child_id in children {
                stack.push((child_id, current_abs_location));
            }
        }
    }

    Ok(())
}

/// 计算并缓存所有视图的 visual_rect
///
/// 在布局完成后调用，缓存每个 view 的 visual_rect()
/// 避免绘制时频繁获取锁和调用虚方法
pub fn compute_visual_rects(start_view_id: ViewId) {
    let child_ids = VIEW_STORAGE.child_ids.read();
    let views = VIEW_STORAGE.views.read();
    let mut visual_rect_cache = VIEW_STORAGE.visual_rect.write();

    let mut stack = vec![start_view_id];

    while let Some(view_id) = stack.pop() {
        // 调用 view 的 visual_rect() 方法（允许子类重写）
        if let Some(view) = views.get(view_id) {
            let (x, y, w, h) = view.read().visual_rect();
            visual_rect_cache.insert(view_id, Rect::new(x, y, w, h));
        }

        // 处理子节点
        if let Some(children) = child_ids.get(view_id) {
            for &child_id in children {
                stack.push(child_id);
            }
        }
    }
}
