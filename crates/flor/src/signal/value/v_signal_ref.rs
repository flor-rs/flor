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
pub enum SignalRef<'a, T: 'static> {
    Const(&'a T),
    Dynamic {
        inner: Ref<'a, Id, Value>,
        _type: PhantomData<T>,
    },
}

impl<'a, T: 'static> SignalRef<'a, T> {
    pub(crate) fn dynamic(inner: Ref<'a, Id, Value>) -> Self {
        Self::Dynamic {
            inner,
            _type: PhantomData,
        }
    }
    pub(crate) fn constant(value: &'a T) -> Self {
        Self::Const(value)
    }
}

impl<'a, T: 'static> Deref for SignalRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            SignalRef::Const(inner) => inner,
            SignalRef::Dynamic { inner, _type } => inner
                .get_ref::<T>()
                .expect("SignalRef: downcast type failed"),
        }
    }
}
