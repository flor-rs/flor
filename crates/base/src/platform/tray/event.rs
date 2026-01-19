#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayEvent {
    /// 鼠标按键按下
    MouseDown(MouseButton),
    /// 鼠标按键松开
    MouseUp(MouseButton),
    /// 鼠标双击
    MouseDoubleClick(MouseButton),
    /// 鼠标进入图标区域 (Windows: NIN_POPUPOPEN)
    MouseEnter,
    /// 鼠标离开图标区域 (Windows: NIN_POPUPCLOSE)
    MouseLeave,
    /// 鼠标在图标上移动
    MouseMove,
}