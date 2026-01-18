use crate::error::Error;
use crate::view::collect_layout_children;
use crate::view::state_selector::CalcTaffyStyle;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::windows::bus::render_from_view_id;
use crate::windows::entry::WindowEntryVisit;
use flor_platform_base::WindowApi;
use log::{trace, warn};
use platform::WindowId;
use std::ops::DerefMut;
use taffy::{AvailableSpace, Point, Size, Style};

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

    Ok(())
}

/// 本地矩形，用于表示控件的边界
/// 保留此类型供可能的其他用途
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl LocalRect {
    pub fn from_layout(size: Size<f32>) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: size.width,
            h: size.height,
        }
    }

    /// 检查矩形是否有效（宽高都大于0）
    pub fn is_valid(&self) -> bool {
        self.w > 0.0 && self.h > 0.0
    }

    pub fn contains(&self, p: Point<f32>) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }
}
