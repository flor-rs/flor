#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowMode {
    /// 普通窗口 (既没最大化也没最小化)
    Normal,
    /// 最小化
    Minimized,
    /// 最大化
    Maximized,
    /// 全屏 (可选，现代App常用)
    Fullscreen,
}
