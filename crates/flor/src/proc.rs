use log::error;
use platform::base::{HandleResult, Message};
use platform::ProcHandler;
use platform::WindowId;

#[derive(Debug, Default)]
pub struct WindowsProcHandler {}
impl ProcHandler for WindowsProcHandler {
    fn window_proc(&self, window_id: WindowId, message: Message) -> HandleResult {
        // 性能监控：测量消息处理时间
        let msg_str = format!("{:?}", message);
        let start = std::time::Instant::now();

        let result = crate::windows::event(window_id, message).unwrap_or_else(|err| {
            error!("bus error: {:?}", err);
            HandleResult::Default
        });

        let elapsed = start.elapsed();
        // 只打印耗时超过 1ms 的消息，避免频繁 I/O
        if elapsed.as_micros() > 1000 {
            let ms = elapsed.as_millis();
            let ns = elapsed.as_nanos() % 1_000_000; // 取纳秒部分（不含毫秒）
            println!("[PERF] {} took {}.{:06} ms", msg_str, ms, ns);
        }

        result
    }
}
