mod batch;
mod constant;
mod create;
mod effect;
mod id;
mod into_read;
mod list;
mod runtime;
mod value;

pub use {
    batch::*, constant::*, create::*, effect::*, id::*, into_read::*, list::*, runtime::*, value::*,
};

pub trait Signal {
    fn id(&self) -> Id;
    /// 信号是否仍存在（未被 destroy），不进行依赖追踪。
    fn exists(&self) -> bool;
    /// 销毁信号，清理存储与订阅。
    fn destroy(&self) {
        RUNTIME.destroy_signal(self.id());
    }
}
