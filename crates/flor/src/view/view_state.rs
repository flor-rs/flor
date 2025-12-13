use crate::view::style::layout::LayoutStateSelector;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use taffy::{Layout, NodeId};

/// 视图状态，存储视图的各种状态数据
pub struct ViewState {
    pub layout: Layout,
    pub node_id: Option<NodeId>,
    pub layout_style: LayoutStateSelector,
    pub dirty_children: bool,
    pub disable: bool,
    pub z_index: i32,
    // 事件处理器
    pub click_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub double_click_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub mouse_enter_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub mouse_leave_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub mouse_move_handler: Option<Arc<dyn Fn(i32, i32) + Send + Sync>>,
    pub focus_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub blur_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub drag_start_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub drag_enter_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub drag_leave_handler: Option<Arc<dyn Fn() + Send + Sync>>,
    pub drop_handler: Option<Arc<dyn Fn() + Send + Sync>>,
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
            click_handler: None,
            double_click_handler: None,
            mouse_enter_handler: None,
            mouse_leave_handler: None,
            mouse_move_handler: None,
            focus_handler: None,
            blur_handler: None,
            drag_start_handler: None,
            drag_enter_handler: None,
            drag_leave_handler: None,
            drop_handler: None,
        }
    }
}
