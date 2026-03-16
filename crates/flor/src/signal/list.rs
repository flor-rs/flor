mod l_read;
mod l_read_signal;
mod l_rw_signal;
mod l_write;
mod l_write_signal;

use crate::signal::{Id, Value};

/// 列表中每个元素的信号单元，带有独立的依赖跟踪 Id
#[derive(Debug)]
pub(crate) struct ListItem {
    pub id: Id,
    pub value: Value,
}

pub use {l_read::*, l_read_signal::*, l_rw_signal::*, l_write::*, l_write_signal::*};
