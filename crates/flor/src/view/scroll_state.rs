#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollState {
    /// 当前滚动位置 (Offset) - 用于渲染变换 translation(-x, -y)
    pub current: (f32, f32),
    /// 最大可滚动范围 (Max Range) - 来自 layout.scroll_width()，用于绘制滚动条滑块占比
    pub max: (f32, f32),
}
