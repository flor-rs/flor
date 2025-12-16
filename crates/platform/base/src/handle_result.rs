#[derive(Default)]
pub enum HandleResult {
    /// 使用系统默认处理函数
    #[default]
    Default,

    /// 告诉系统，我处理了，返回0
    Handled,

    /// 窗口关闭处理消息
    /// true 代表允许窗口正常关闭
    /// false 阻止窗口进行关闭
    WindowClose(bool),
}
