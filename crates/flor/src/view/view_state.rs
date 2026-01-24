use crate::view::resolver::LayoutResolver;
use std::fmt::{Debug, Formatter};
use taffy::{Layout, NodeId};

/// 视图状态，存储视图的各种状态数据
pub struct ViewState {
    pub layout: Layout,
    /// 绝对位置（相对于窗口左上角），由 bus_update_layout 计算
    pub abs_location: (f32, f32),
    pub node_id: Option<NodeId>,
    pub layout_style: LayoutResolver,
    pub dirty_children: bool,
    pub disable: bool,
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
            abs_location: (0.0, 0.0),
            node_id: None,
            layout_style: LayoutResolver::new(),
            dirty_children: false,
            disable: false,
        }
    }
}
