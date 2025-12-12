use crate::signal::id::Id;
use crate::signal::runtime::RUNTIME;
use rustc_hash::FxHashSet;
use std::cell::RefCell;

thread_local! {
    pub static BATCH: RefCell<Batch> = RefCell::new(Batch::default());
}

#[derive(Default)]
pub struct Batch {
    pub(crate) batch_mode: bool,
    pub(crate) signal_effect_ids: FxHashSet<Id>,
}

/// 批量执行操作，将闭包内的信号更新延迟触发副作用。
///
/// - 闭包内的信号写入不会立即触发订阅的副作用。
/// - 所有更新在闭包执行完毕后，统一触发一次副作用（去重）。
/// - **线程隔离**：批处理只影响当前线程，跨线程的信号更新不会被捕获到当前批处理中。
pub fn batch(f: impl Fn()) {
    // 先设置 batch_mode
    BATCH.with(|batch| batch.borrow_mut().batch_mode = true);

    // 执行闭包
    f();

    // 批量收集并清理 signal_effect_ids
    let collected_signal_ids = BATCH.with(|batch| {
        let mut b = batch.borrow_mut();
        b.batch_mode = false;
        let collected_signal_ids = b.signal_effect_ids.iter().copied().collect::<Vec<_>>();
        b.signal_effect_ids.clear();
        collected_signal_ids
    });

    // 批量触发副作用
    RUNTIME.run_effects_for_signals(collected_signal_ids);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::effect::effect::create_effect;
    use crate::signal::read::Read;
    use crate::signal::rw_signal::create_signal;
    use crate::signal::write::Write;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_batch_update_single_thread() {
        let (r, w) = create_signal(0).split();

        let log = Rc::new(RefCell::new(vec![]));
        let log_clone = Rc::clone(&log);

        create_effect(move |_| {
            let x = r.get();
            let mut log = log_clone.borrow_mut();
            log.push(x);
        });

        // 单个 set 立即触发
        w.set(1);
        assert_eq!(&*log.borrow(), &[0, 1]);

        // batch 内多次 set
        batch(|| {
            w.set(2);
            w.set(3);
            w.set(4);
            // 此时 effect 不应触发
            assert_eq!(&*log.borrow(), &[0, 1]);
        });

        // batch 完成后 effect 应该只触发一次
        assert_eq!(&*log.borrow(), &[0, 1, 4]);
    }

    #[test]
    fn test_batch_update_multi_thread() {
        let (r, w) = create_signal(0).split();

        let log = Arc::new(Mutex::new(vec![]));
        let log_clone = Arc::clone(&log);

        create_effect(move |_| {
            let val = r.get();
            log_clone.lock().unwrap().push(val);
        });

        // 多线程 batch
        let handles: Vec<_> = (1..=5)
            .map(|i| {
                let w = w;
                thread::spawn(move || {
                    batch(|| {
                        w.set(i);
                    });
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let final_val = r.get();
        // 最终信号值是最后一次 set 的值
        assert!(final_val >= 1 && final_val <= 5);

        let logs = log.lock().unwrap();
        // 每个线程的 batch 内只触发一次
        assert_eq!(logs.len(), 6); // 初始 0 + 5 个线程 batch 后触发
    }
}
