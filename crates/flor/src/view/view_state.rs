use crate::view::style::layout::LayoutStateSelector;
use std::fmt::{Debug, Formatter};
use taffy::{Layout, NodeId};

/// 视图状态，存储视图的各种状态数据
pub struct ViewState {
    pub layout: Layout,
    pub node_id: Option<NodeId>,
    pub layout_style: LayoutStateSelector,
    pub dirty_children: bool,
    pub disable: bool,
    pub z_index: i32,
}

impl Debug for ViewState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.layout.fmt(f)?;
        self.node_id.fmt(f)?;
        self.layout_style.fmt(f)?;
        Ok(())
    }
}

impl ViewState {
    pub fn new() -> ViewState {
        ViewState {
            layout: Layout::default(),
            node_id: None,
            layout_style: LayoutStateSelector::default(),
            dirty_children: false,
            disable: false,
            z_index: 0,
        }
    }
}
