use log::{error, warn};
use std::backtrace::Backtrace;

// 1. Trait 名字改为 ResultLogExt (Result Log Extension)，这是 Rust 扩展 Trait 的标准命名惯例
pub trait ResultLogExt<E> {
    fn error_on_err(self, msg: impl AsRef<str>);
    fn warn_on_err(self, msg: impl AsRef<str>);

    fn log_err(self, msg: impl AsRef<str>) -> Self;
    fn log_warn(self, msg: impl AsRef<str>) -> Self;
}

impl<T, E: std::fmt::Debug> ResultLogExt<E> for Result<T, E> {
    fn error_on_err(self, msg: impl AsRef<str>) {
        if let Err(err) = self.as_ref() {
            let bt = Backtrace::capture();
            // 这里 msg 放在前面，err 放在后面，阅读体验更好： "Load Config Failed: File not found"
            error!("{}: {:?} \n{}", msg.as_ref(), err, bt);
        }
    }

    fn warn_on_err(self, msg: impl AsRef<str>) {
        if let Err(err) = self.as_ref() {
            // 修正：这里应该用 warn! 宏
            warn!("{}: {:?}", msg.as_ref(), err);
        }
    }

    // --- 新增链式实现 ---
    fn log_err(self, msg: impl AsRef<str>) -> Self {
        if let Err(err) = &self {
            let bt = Backtrace::capture();
            // 使用 &self 借用打印，不消耗所有权
            error!("{}: {:?}\n{}", msg.as_ref(), err, bt);
        }
        self // 原样返回
    }

    fn log_warn(self, msg: impl AsRef<str>) -> Self {
        if let Err(err) = &self {
            warn!("{}: {:?}", msg.as_ref(), err);
        }
        self // 原样返回
    }
}
