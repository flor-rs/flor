use crate::signal::{Id, Value};
use dashmap::mapref::one::{Ref, RefMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

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

/// 信号的可变引用守卫。
///
/// 持有 DashMap 底层分片的写锁，实现了 [`DerefMut<Target = T>`]，
/// 可以像普通 `&mut T` 一样透明使用。
///
/// 注意：守卫 drop 时不会自动触发信号订阅的 Effect；
/// 如需触发，请在修改后调用 `set`/`update`/`try_set`/`try_update`。
/// 持有此守卫期间，不要在同一线程上对同一信号调用 `set`/`update`，
/// 否则会尝试再次获取写锁导致死锁。
///
/// 通过 [`Write::update`] 或 [`Write::try_update`] 间接获取内部可变引用，
/// 请勿跨异步边界持有此守卫。
pub struct SignalRefMut<'a, T: 'static> {
    id: Id,
    inner: RefMut<'a, Id, Value>,
    _type: PhantomData<T>,
}

impl<'a, T: 'static> SignalRefMut<'a, T> {
    pub(crate) fn new(id: Id, inner: RefMut<'a, Id, Value>) -> Self {
        Self {
            id,
            inner,
            _type: PhantomData,
        }
    }
}

impl<'a, T: 'static> Deref for SignalRefMut<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.inner
            .get_ref::<T>()
            .expect("SignalRefMut: downcast type failed")
    }
}

impl<'a, T: 'static> DerefMut for SignalRefMut<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.inner
            .value_mut()
            .get_mut_ref::<T>()
            .expect("SignalRefMut: downcast type failed")
    }
}
