#[derive(Default, PartialEq, Eq, Debug, Copy, Clone)]
pub enum HandleResult {
    /// 使用系统默认处理函数
    #[default]
    Default,

    /// 告诉系统，我处理了，返回0
    Handled,
}
