//! 命中测试入口
//!
//! 设计思路：
//! 1. 使用绝对坐标 (abs_location) 进行命中测试，与 draw_entry 保持一致
//! 2. 两阶段命中：先检查 overlay 层（滚动条等），再检查 main 层
//! 3. 递归深度优先，后绘制的元素先检测（z-order：最上层优先）
//! 4. 正确处理 clip 和 scroll offset
//!
//! 关键点：
//! - abs_location 是"逻辑位置"（不考虑 scroll）
//! - 在绘制时，scroll 是通过 transform 来实现的
//! - 在命中测试时，需要累积祖先的 scroll offset 来计算"可视位置"

use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use flor_base::platform::{KeyState, MousePosition};
use platform::WindowId;
use taffy::{Display, Overflow, Point};

/// 命中测试结果
#[derive(Debug, Clone, Copy)]
pub struct HitTestResult {
    pub view_id: ViewId,
    pub is_overlay: bool,
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

    // 执行命中测试
    // accumulated_scroll 表示从根节点到当前节点累积的滚动偏移
    if let Some(result) = hit_test_recursive(
        root_id,
        mouse_point,
        key_state,
        (0.0, 0.0), // 根节点没有累积的滚动偏移
        None,       // 根节点没有 clip
        &views,
        &child_ids,
        &states,
        &visual,
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
/// - `mouse_point`: 鼠标位置（相对于窗口）
/// - `accumulated_scroll`: 从根节点到当前节点累积的滚动偏移
/// - `parent_clip`: 父节点的 clip 区域
///
/// # 返回
/// - Some(result): 命中了某个节点
/// - None: 该分支没有命中任何节点
fn hit_test_recursive(
    view_id: ViewId,
    mouse_point: (f32, f32),
    key_state: KeyState,
    accumulated_scroll: (f32, f32),
    parent_clip: Option<ClipRect>,
    views: &slotmap::SecondaryMap<ViewId, parking_lot::RwLock<Box<dyn crate::view::View + Send + Sync>>>,
    child_ids: &slotmap::SecondaryMap<ViewId, Vec<ViewId>>,
    states: &slotmap::SecondaryMap<ViewId, parking_lot::RwLock<crate::view::view_state::ViewState>>,
    visual: &slotmap::SecondaryMap<ViewId, bool>,
) -> Option<HitTestResult> {
    // 1. 检查可见性
    if let Some(&is_visual) = visual.get(view_id) {
        if !is_visual {
            return None;
        }
    }

    // 2. 获取节点状态
    let state_lock = states.get(view_id)?;
    let state = state_lock.read();

    // 3. 检查 display: none
    let style = view_id.calc_current_style().ok()?;
    if style.display == Display::None {
        return None;
    }

    // 4. 获取节点的绝对位置和尺寸
    let abs_location = state.abs_location;
    let layout_size = state.layout.size;

    // 5. 计算"可视位置" = 逻辑位置 - 累积的滚动偏移
    // 这是因为绘制时使用 transform(-scroll) 来偏移内容
    let visual_location = (
        abs_location.0 - accumulated_scroll.0,
        abs_location.1 - accumulated_scroll.1,
    );

    // 6. 计算当前节点的可视边界矩形
    let node_rect = Rect {
        x: visual_location.0,
        y: visual_location.1,
        w: layout_size.width,
        h: layout_size.height,
    };

    // 7. 计算当前节点的 clip 区域（如果有 overflow 设置）
    let current_clip = if style.overflow != Point::<Overflow>::default() {
        let mut clip = ClipRect {
            left: f32::NEG_INFINITY,
            top: f32::NEG_INFINITY,
            right: f32::INFINITY,
            bottom: f32::INFINITY,
        };

        if style.overflow.x != Overflow::Visible {
            clip.left = visual_location.0;
            clip.right = visual_location.0 + layout_size.width;
        }
        if style.overflow.y != Overflow::Visible {
            clip.top = visual_location.1;
            clip.bottom = visual_location.1 + layout_size.height;
        }

        // 与父级 clip 取交集
        Some(clip.intersect(parent_clip))
    } else {
        parent_clip
    };

    // 8. 如果鼠标不在 clip 区域内，整个分支都可以跳过（剪枝优化）
    if let Some(clip) = current_clip {
        if !clip.contains(mouse_point) {
            return None;
        }
    }

    // 9. 获取当前节点的滚动偏移
    let scroll_offset = view_id.scroll_offset().unwrap_or((0.0, 0.0));

    // 释放 state 锁，避免死锁
    drop(state);

    // 10. 检查 overlay 层（滚动条等）
    // overlay 不受 scroll 影响，使用可视位置
    if let Some(view_lock) = views.get(view_id) {
        let view = view_lock.read();
        if view.on_hit_test_overlay(
            MousePosition {
                x: mouse_point.0 as i32,
                y: mouse_point.1 as i32,
            },
            key_state,
        ) {
            return Some(HitTestResult {
                view_id,
                is_overlay: true,
            });
        }
    }

    // 11. 计算传递给子节点的累积滚动偏移
    // 子节点的内容会被当前节点的 scroll offset 影响
    let child_accumulated_scroll = (
        accumulated_scroll.0 + scroll_offset.0,
        accumulated_scroll.1 + scroll_offset.1,
    );

    // 12. 检查子节点（反向遍历：最后绘制的最先检测）
    if let Some(children) = child_ids.get(view_id) {
        for &child_id in children.iter().rev() {
            if let Some(result) = hit_test_recursive(
                child_id,
                mouse_point,
                key_state,
                child_accumulated_scroll,
                current_clip,
                views,
                child_ids,
                states,
                visual,
            ) {
                return Some(result);
            }
        }
    }

    // 13. 检查自身（main 层）
    // 只有当鼠标在节点可视边界内时才进行具体的命中测试
    if node_rect.contains(mouse_point) {
        if let Some(view_lock) = views.get(view_id) {
            let view = view_lock.read();
            if view.on_hit_test(
                MousePosition {
                    x: mouse_point.0 as i32,
                    y: mouse_point.1 as i32,
                },
                key_state,
            ) {
                return Some(HitTestResult {
                    view_id,
                    is_overlay: false,
                });
            }
        }
    }

    None
}

/// 矩形（用于边界检查）
#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn contains(&self, point: (f32, f32)) -> bool {
        point.0 >= self.x
            && point.0 <= self.x + self.w
            && point.1 >= self.y
            && point.1 <= self.y + self.h
    }
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
