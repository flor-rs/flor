use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::render::FlorRenderer;
use crate::view::{ControlState, View, ViewId, VIEW_STORAGE};
use crate::windows::WindowEntryVisit;
use flor_base::graphics::RenderContext;
use flor_base::types::{Rect, Transform2D};
use log::debug;
use platform::WindowId;
use taffy::{Display, Layout, Overflow, Point};

enum DrawStage {
    Enter,
    Exit {
        transform_depth: u32,
        clip_depth: Option<u32>,
        layout: Layout,
    },
}

struct DrawFrame {
    view_id: ViewId,
    abs_location: (f32, f32),
    /// 当前的裁剪区域（绝对坐标），用于同层剪枝
    /// None 表示没有裁剪限制（全部可见）
    clip_rect: Option<Rect<f32, f32>>,
    stage: DrawStage,
}

pub fn draw_entry(window_id: WindowId, render: &mut FlorRenderer) -> Result<(), Error> {
    let views = VIEW_STORAGE.views.read();
    let child_map = VIEW_STORAGE.child_ids.read();
    let view_transform = VIEW_STORAGE.transform.read();
    let mut visuals = VIEW_STORAGE.visual.write();
    let states = VIEW_STORAGE.states.read();
    let pressed = VIEW_STORAGE.pressed.read();
    let visual_rect_cache = VIEW_STORAGE.visual_rect.read();
    let Some(window_entry) = window_id.entry() else {
        return Ok(());
    };

    let mut stack = Vec::with_capacity(64);

    // 根控件没有裁剪限制
    stack.push(DrawFrame {
        view_id: window_id.view_id(),
        abs_location: (0.0, 0.0),
        clip_rect: None,
        stage: DrawStage::Enter,
    });

    let mut drawn_count = 0;
    let mut culled_count = 0;

    visuals.clear();
    while let Some(frame) = stack.pop() {
        match frame.stage {
            DrawStage::Enter => {
                let view_id = frame.view_id;

                let Some(view) = views.get(view_id) else {
                    continue;
                };

                // 1. State & Layout Lookup
                let Some(view_state) = states.get(view_id).map(|s| s.read()) else {
                    culled_count += 1;
                    continue;
                };
                let layout = view_state.layout;

                let abs_location = (
                    frame.abs_location.0 + layout.location.x,
                    frame.abs_location.1 + layout.location.y,
                );

                // 2. Culling (使用缓存的 visual_rect)
                if let Some(ref parent_clip) = frame.clip_rect {
                    if let Some(visual_rect) = visual_rect_cache.get(view_id) {
                        if !visual_rect.intersects(parent_clip) {
                            culled_count += 1;
                            continue;
                        }
                    }
                }

                // 3. Style Calculation（按优先级：Disabled > Active > Focus > Hover > Normal）
                let control_state = match true {
                    _ if view_state.disable => ControlState::Disabled,
                    _ if pressed.get(view_id).is_some() => ControlState::Active,
                    _ if window_entry.focus_manager.is_focused(view_id) => ControlState::Focus,
                    _ if window_entry.hover_id == Some(view_id) => ControlState::Hover,
                    _ => ControlState::Normal,
                };

                let style = view_state.layout_style.get_data_borrow(control_state);

                if style.display == Display::None {
                    culled_count += 1;
                    continue;
                }

                // Setup Overhead
                let transform_depth = render.get_transform_depth()?;
                let mut clip_depth = None;
                let mut new_clip_rect: Option<Rect<f32, f32>> = None;

                if let Some(transform) = view_transform.get(view_id) {
                    render.push_transform(transform)?;
                }

                if style.overflow != Point::<Overflow>::default() {
                    clip_depth = Some(render.get_clip_depth()?);

                    const INF: f32 = 1_000_000_000.0;
                    let mut clip_x = -INF;
                    let mut clip_y = -INF;
                    let mut clip_w = INF * 2.0;
                    let mut clip_h = INF * 2.0;

                    if style.overflow.x != Overflow::Visible {
                        clip_x = abs_location.0;
                        clip_w = layout.size.width;
                    }
                    if style.overflow.y != Overflow::Visible {
                        clip_y = abs_location.1;
                        clip_h = layout.size.height;
                    }

                    new_clip_rect = Some(Rect::new(clip_x, clip_y, clip_w, clip_h));
                }

                // Draw
                visuals.insert(view_id, ());
                view.write()
                    .on_draw(render, abs_location, layout)
                    .error_on_err(format!("view id: {}", view_id));
                drawn_count += 1;

                // Post-Draw Setup (Child clipping etc)
                if let Some(ref clip_rect) = new_clip_rect {
                    render.push_clip(clip_rect.to_tuple())?;
                }

                let mut child_clip_rect = match (new_clip_rect, frame.clip_rect) {
                    (Some(new_clip), Some(parent_clip)) => new_clip.intersection(&parent_clip),
                    (Some(new_clip), None) => Some(new_clip),
                    (None, Some(parent_clip)) => Some(parent_clip),
                    (None, None) => None,
                };

                if let Some((scroll_x, scroll_y)) = view_id.scroll_offset() {
                    if scroll_x != 0.0 || scroll_y != 0.0 {
                        render.push_transform(&Transform2D::translation(-scroll_x, -scroll_y))?;
                        if let Some(ref clip) = child_clip_rect {
                            child_clip_rect = Some(clip.translate(scroll_x, scroll_y));
                        }
                    }
                }

                stack.push(DrawFrame {
                    view_id,
                    abs_location,
                    clip_rect: child_clip_rect,
                    stage: DrawStage::Exit {
                        transform_depth,
                        clip_depth,
                        layout,
                    },
                });

                if let Some(children) = child_map.get(view_id) {
                    for &child_id in children.iter() {
                        stack.push(DrawFrame {
                            view_id: child_id,
                            abs_location,
                            clip_rect: child_clip_rect,
                            stage: DrawStage::Enter,
                        });
                    }
                }
            }

            DrawStage::Exit {
                transform_depth,
                clip_depth,
                layout,
            } => {
                let Some(view) = views.get(frame.view_id) else {
                    continue;
                };

                render.pop_transform(Some(transform_depth))?;
                if clip_depth.is_some() {
                    render.pop_clip(clip_depth)?;
                }

                view.write()
                    .on_draw_overlay(render, frame.abs_location, layout)?;
            }
        }
    }

    debug!("draw stats: drawn={} culled={}", drawn_count, culled_count);
    Ok(())
}
