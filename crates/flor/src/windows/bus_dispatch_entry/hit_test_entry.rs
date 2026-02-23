//! 命中测试入口
//!
//! 设计思路：
//! 1. 使用 accumulated_transform 进行坐标变换
//!    - accumulated_transform 代表：控件局部坐标(0,0) → 窗口坐标
//!    - 命中测试时用逆变换把鼠标窗口坐标转为控件局部坐标
//! 2. 两阶段命中：先检查 overlay 层（滚动条等），再检查 main 层
//! 3. 递归深度优先，后绘制的元素先检测（z-order：最上层优先）
//! 4. 正确处理 clip

use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use flor_base::platform::{KeyState, MousePosition};
use flor_base::types::Transform2D;
use platform::WindowId;
use taffy::{Display, Overflow, Point};

/// 命中测试结果
#[derive(Debug, Clone, Copy)]
pub struct HitTestResult {
    pub view_id: ViewId,
}

/// 命中测试的主入口
///
/// # 参数
/// - `window_id`: 窗口 ID
/// - `mouse_pos`: 鼠标位置（相对于窗口客户区）
/// - `key_state`: 键盘状态
///
/// # 返回
/// 命中的 ViewId（如果没有命中任何元素，返回根节点）
pub fn hit_test_entry(
    window_id: WindowId,
    mouse_pos: MousePosition,
    key_state: KeyState,
) -> ViewId {
    let root_id = window_id.view_id();
    let mouse_point = (mouse_pos.x as f32, mouse_pos.y as f32);

    // 获取全局锁
    let views = VIEW_STORAGE.views.read();
    let child_ids = VIEW_STORAGE.child_ids.read();
    let states = VIEW_STORAGE.states.read();
    let visual = VIEW_STORAGE.visual.read();
    let accumulated_transform = VIEW_STORAGE.accumulated_transform.read();

    // 执行命中测试
    if let Some(result) = hit_test_recursive(
        root_id,
        mouse_point,
        key_state,
        None, // 根节点没有 clip
        &views,
        &child_ids,
        &states,
        &visual,
        &accumulated_transform,
    ) {
        result.view_id
    } else {
        root_id
    }
}

/// 递归命中测试
///
/// # 参数
/// - `view_id`: 当前节点
/// - `mouse_point`: 鼠标位置（相对于窗口，窗口坐标系）
/// - `parent_clip`: 父节点的 clip 区域（窗口坐标系）
///
/// # 返回
/// - Some(result): 命中了某个节点
/// - None: 该分支没有命中任何节点
fn hit_test_recursive(
    view_id: ViewId,
    mouse_point: (f32, f32),
    key_state: KeyState,
    parent_clip: Option<ClipRect>,
    views: &slotmap::SecondaryMap<
        ViewId,
        parking_lot::RwLock<Box<dyn crate::view::View + Send + Sync>>,
    >,
    child_ids: &slotmap::SecondaryMap<ViewId, Vec<ViewId>>,
    states: &slotmap::SecondaryMap<ViewId, parking_lot::RwLock<crate::view::view_state::ViewState>>,
    visual: &slotmap::SecondaryMap<ViewId, ()>,
    accumulated_transform: &slotmap::SecondaryMap<ViewId, Transform2D>,
) -> Option<HitTestResult> {
    // 1. 检查可见性
    if visual.get(view_id).is_none() {
        return None;
    }

    // 2. 获取节点状态
    let state_lock = states.get(view_id)?;
    let state = state_lock.read();

    // 3. 检查 display: none
    let style = state.layout_style.get_data_borrow(view_id.control_state());
    if style.display == Display::None {
        return None;
    }

    // 4. 获取控件尺寸
    let layout_size = state.layout.size;

    // 5. 获取累积变换，把鼠标窗口坐标转为控件局部坐标
    let local_mouse_point = if let Some(transform) = accumulated_transform.get(view_id) {
        if let Some(local) = transform.inverse_transform_point(mouse_point.0, mouse_point.1) {
            local
        } else {
            // 变换不可逆（如缩放为0），跳过
            return None;
        }
    } else {
        // 没有累积变换，使用原始鼠标坐标
        // 这不应该发生，但作为后备
        mouse_point
    };

    // 6. 计算当前节点的 clip 区域（窗口坐标系）
    // 注意：clip 区域需要在窗口坐标系中计算，因为鼠标坐标也是窗口坐标
    let current_clip = if style.overflow != Point::<Overflow>::default() {
        // 将控件的四个角变换到窗口坐标系，计算轴对齐包围盒
        let transform = accumulated_transform
            .get(view_id)
            .copied()
            .unwrap_or(Transform2D::IDENTITY);

        let mut clip = ClipRect {
            left: f32::NEG_INFINITY,
            top: f32::NEG_INFINITY,
            right: f32::INFINITY,
            bottom: f32::INFINITY,
        };

        if style.overflow.x != Overflow::Visible || style.overflow.y != Overflow::Visible {
            // 变换控件边界到窗口坐标系
            let (min_x, min_y, w, h) =
                transform.transform_rect(0.0, 0.0, layout_size.width, layout_size.height);

            if style.overflow.x != Overflow::Visible {
                clip.left = min_x;
                clip.right = min_x + w;
            }
            if style.overflow.y != Overflow::Visible {
                clip.top = min_y;
                clip.bottom = min_y + h;
            }
        }

        // 与父级 clip 取交集
        Some(clip.intersect(parent_clip))
    } else {
        parent_clip
    };

    // 7. 如果鼠标不在 clip 区域内（窗口坐标系检查），整个分支都可以跳过
    if let Some(clip) = current_clip {
        if !clip.contains(mouse_point) {
            return None;
        }
    }

    // 8. 检查 overlay 层（滚动条等）
    // overlay 使用控件局部坐标
    if let Some(view_lock) = views.get(view_id) {
        let view = view_lock.read();
        if view.on_hit_test_overlay(
            MousePosition {
                x: local_mouse_point.0 as i32,
                y: local_mouse_point.1 as i32,
            },
            key_state,
        ) {
            return Some(HitTestResult { view_id });
        }
    }

    // 9. 检查子节点（反向遍历：最后绘制的最先检测）
    if let Some(children) = child_ids.get(view_id) {
        for &child_id in children.iter().rev() {
            if let Some(result) = hit_test_recursive(
                child_id,
                mouse_point, // 传递原始窗口坐标，每个子节点用自己的 accumulated_transform
                key_state,
                current_clip,
                views,
                child_ids,
                states,
                visual,
                accumulated_transform,
            ) {
                return Some(result);
            }
        }
    }

    // 10. 检查自身（main 层）
    // 使用控件局部坐标检查是否在 (0, 0, width, height) 范围内
    let in_bounds = local_mouse_point.0 >= 0.0
        && local_mouse_point.0 <= layout_size.width
        && local_mouse_point.1 >= 0.0
        && local_mouse_point.1 <= layout_size.height;

    if in_bounds {
        if let Some(view_lock) = views.get(view_id) {
            let view = view_lock.read();
            if view.on_hit_test(
                MousePosition {
                    x: local_mouse_point.0 as i32,
                    y: local_mouse_point.1 as i32,
                },
                key_state,
            ) {
                return Some(HitTestResult { view_id });
            }
        }
    }

    None
}

/// 裁剪区域
#[derive(Clone, Copy, Debug)]
struct ClipRect {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl ClipRect {
    fn contains(&self, point: (f32, f32)) -> bool {
        point.0 >= self.left
            && point.0 <= self.right
            && point.1 >= self.top
            && point.1 <= self.bottom
    }

    /// 与另一个 clip 区域取交集
    fn intersect(self, other: Option<ClipRect>) -> ClipRect {
        match other {
            Some(o) => ClipRect {
                left: self.left.max(o.left),
                top: self.top.max(o.top),
                right: self.right.min(o.right),
                bottom: self.bottom.min(o.bottom),
            },
            None => self,
        }
    }
}
