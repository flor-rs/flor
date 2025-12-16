/// 跨平台光标图标的枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorIcon {
    /// 默认光标 (通常是箭头)。
    Default,
    /// 文本输入光标 (I 型梁)。
    Text,
    /// 指针/链接光标 (手型)。
    Pointer,
    /// 正在等待/工作光标 (沙漏/旋转圈)。
    Wait,
    /// 忙碌光标 (结合了 Default 和 Wait)。
    Crosshair,
    /// 移动/抓取光标 (四个方向的箭头)。
    Move,
    /// 抓取光标 (打开的手)。
    Grab,
    /// 正在抓取光标 (闭合的手)。
    Grabbing,
    /// 不允许/禁用光标 (圆圈和斜杠)。
    NotAllowed,
    // 调整大小光标 (Resize)
    NResize,    // North
    SResize,    // South
    EResize,    // East
    WResize,    // West
    NeResize,   // North-East
    NwResize,   // North-West
    SeResize,   // South-East
    SwResize,   // South-West
    NsResize,   // North-South
    EwResize,   // East-West
    NeswResize, // North-East/South-West diagonal
    NwseResize, // North-West/South-East diagonal
    // 其他如：
    ContextMenu,
    Help,
    Progress,
    Cell,
    Alias,
    Copy,
}
