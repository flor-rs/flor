use log::{error, warn};

// 1. Trait 名字改为 ResultLogExt (Result Log Extension)，这是 Rust 扩展 Trait 的标准命名惯例
pub trait ResultLogExt<E> {
    fn error_on_err(self, msg: impl AsRef<str>);
    fn warn_on_err(self, msg: impl AsRef<str>);
}

impl<T, E: std::fmt::Debug> ResultLogExt<E> for Result<T, E> {
    // 加上 #[track_caller] 是个好习惯，这样日志里显示的文件行号
    // 会是调用这行代码的地方，而不是这个 Trait 实现的地方。
    #[track_caller]
    fn error_on_err(self, msg: impl AsRef<str>) {
        if let Err(err) = self.as_ref() {
            // 这里 msg 放在前面，err 放在后面，阅读体验更好： "Load Config Failed: File not found"
            error!("{}: {:?}", msg.as_ref(), err);
        }
    }

    #[track_caller]
    fn warn_on_err(self, msg: impl AsRef<str>) {
        if let Err(err) = self.as_ref() {
            // 修正：这里应该用 warn! 宏
            warn!("{}: {:?}", msg.as_ref(), err);
        }
    }
}
