use crate::signal::{Signal, SignalRef, RUNTIME, SCOPE};

pub trait Read<T>: Signal {
    /// 跟踪
    fn track(&self) {
        if let Some(scope_id) = SCOPE.get() {
            RUNTIME.subscribe(self.id(), scope_id);
        }
    }

    fn try_get(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.track();
        RUNTIME.values.get(&self.id())?.get::<T>()
    }

    fn get(&self) -> T
    where
        T: Clone + 'static,
    {
        self.track();
        RUNTIME
            .values
            .get(&self.id())
            .expect(
                "invalid signal id: this signal has likely been destroyed.\n\
                    Signals in this system may be manually cleaned up or reclaimed.\n\
                    If the signal lifetime is not guaranteed, use `try_get()` instead of `get()`.",
            )
            .get::<T>()
            .expect("to downcast signal type fail")
            .clone()
    }

    /// 返回一个持有 DashMap 读锁的不可变引用守卫 [`SignalRef`] 并进行依赖追踪。
    ///
    /// 守卫实现了 `Deref<Target = T>`，可像普通 `&T` 使用。
    /// 守卫 drop 时锁才释放，适合需要跨多条语句持有引用的场景。
    ///
    /// # 注意
    /// 持有守卫期间，同一线程对同一信号调用 `set`/`update`/`get_mut`/`get_mut_ref` 会死锁。
    fn get_ref(&self) -> SignalRef<'_, T> {
        self.track();
        let guard = RUNTIME.values.get(&self.id()).expect(
            "invalid signal id: this signal has likely been destroyed.\n\
                    If the signal lifetime is not guaranteed, use `try_get_ref()` instead.",
        );
        SignalRef::dynamic(guard)
    }

    /// 尝试返回持有 DashMap 读锁的不可变引用守卫 [`SignalRef`]。
    /// 如果信号已被销毁，返回 `None`。
    fn try_get_ref(&self) -> Option<SignalRef<'_, T>> {
        self.track();
        let guard = RUNTIME.values.get(&self.id())?;
        Some(SignalRef::dynamic(guard))
    }
}
