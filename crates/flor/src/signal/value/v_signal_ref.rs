use crate::signal::{Id, Value};
use dashmap::mapref::one::Ref;
use std::marker::PhantomData;
use std::ops::Deref;

/// 信号的不可变引用守卫。
///
/// 持有 DashMap 底层分片的读锁，实现了 [`Deref<Target = T>`]，
/// 可以像普通 `&T` 一样透明使用。
/// 当此守卫 drop 时，读锁自动释放。
///
/// 通过 [`Read::get_ref`] 或 [`Read::try_get_ref`] 获取。
pub struct SignalRef<'a, T: 'static> {
    inner: Ref<'a, Id, Value>,
    _type: PhantomData<T>,
}

impl<'a, T: 'static> SignalRef<'a, T> {
    pub(crate) fn new(inner: Ref<'a, Id, Value>) -> Self {
        Self {
            inner,
            _type: PhantomData,
        }
    }
}

impl<'a, T: 'static> Deref for SignalRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.inner
            .get_ref::<T>()
            .expect("SignalRef: downcast type failed")
    }
}
