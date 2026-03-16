use crate::signal::list::ListItem;
use crate::signal::{Id, Signal, RUNTIME, SCOPE};
use dashmap::mapref::one::Ref;
use std::marker::PhantomData;

pub trait ListRead<T>: Signal {
    /// 跟踪
    fn track(&self) {
        if let Some(scope_id) = SCOPE.get() {
            RUNTIME.subscribe(self.id(), scope_id);
        }
    }

    /// 返回列表长度；列表不存在时返回 None。
    fn len(&self) -> Option<usize> {
        self.track();
        RUNTIME.list_signal.get(&self.id()).map(|guard| guard.len())
    }

    /// 返回列表长度；不存在时返回 0。
    fn len_or_zero(&self) -> usize {
        self.len().unwrap_or(0)
    }

    /// 是否为空；列表不存在时视为 0 返回 true。
    fn is_empty(&self) -> bool {
        self.len_or_zero() == 0
    }

    /// 判断列表中是否包含等于给定值的元素，不存在列表则 panic。
    fn contains(&self, value: &T) -> bool
    where
        T: PartialEq + 'static,
    {
        self.try_contains(value)
            .expect("invalid list signal id: this signal has likely been destroyed")
    }

    /// 尝试判断列表中是否包含等于给定值的元素；列表不存在时返回 None。
    /// 会对列表进行结构订阅，并对迭代到的行进行行级订阅，以在元素值被 set/update 时重新计算。
    fn try_contains(&self, value: &T) -> Option<bool>
    where
        T: PartialEq + 'static,
    {
        self.track(); // 订阅列表 Id（结构变化）
        let guard = RUNTIME.list_signal.get(&self.id())?;
        for item in guard.iter() {
            if let Some(scope_id) = SCOPE.get() {
                RUNTIME.subscribe(item.id, scope_id); // 订阅行值变化
            }
            if let Some(v) = item.value.get_ref::<T>() {
                if v == value {
                    return Some(true);
                }
            }
        }
        Some(false)
    }

    /// 尝试获取指定位置的元素
    fn try_get(&self, index: usize) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.track();
        let guard = RUNTIME.list_signal.get(&self.id())?;
        let item: &ListItem = guard.get(index)?;
        if let Some(scope_id) = SCOPE.get() {
            RUNTIME.subscribe(item.id, scope_id);
        }
        item.value.get::<T>()
    }

    /// 获取指定位置元素，不存在时 panic
    fn get(&self, index: usize) -> T
    where
        T: Clone + 'static,
    {
        self.track();
        let guard = RUNTIME
            .list_signal
            .get(&self.id())
            .expect("invalid list signal id: this signal has likely been destroyed");
        let item = guard.get(index).expect("list index out of bounds");
        if let Some(scope_id) = SCOPE.get() {
            RUNTIME.subscribe(item.id, scope_id);
        }
        item.value
            .get::<T>()
            .expect("to downcast list item type fail")
    }

    /// 克隆为 Vec
    fn to_vec(&self) -> Vec<T>
    where
        T: Clone + 'static,
    {
        self.track();
        let guard = RUNTIME
            .list_signal
            .get(&self.id())
            .expect("invalid list signal id: this signal has likely been destroyed");
        guard
            .iter()
            .map(|item| {
                if let Some(scope_id) = SCOPE.get() {
                    RUNTIME.subscribe(item.id, scope_id);
                }
                item.value
                    .get::<T>()
                    .expect("to downcast list item type fail")
            })
            .collect()
    }

    /// 尝试克隆为 Vec
    fn try_to_vec(&self) -> Option<Vec<T>>
    where
        T: Clone + 'static,
    {
        self.track();
        let guard = RUNTIME.list_signal.get(&self.id())?;
        let mut out = Vec::with_capacity(guard.len());
        if let Some(scope_id) = SCOPE.get() {
            for item in guard.iter() {
                RUNTIME.subscribe(item.id, scope_id);
                out.push(
                    item.value
                        .get::<T>()
                        .expect("try_to_vec: to downcast list item type fail"),
                );
            }
        } else {
            for item in guard.iter() {
                out.push(
                    item.value
                        .get::<T>()
                        .expect("try_to_vec: to downcast list item type fail"),
                );
            }
        }
        Some(out)
    }

    /// 返回一个持有 DashMap 读锁的迭代器，逐行读取，避免整表 clone。
    /// 如果信号不存在返回 None。
    fn iter(&self) -> Option<ListIter<'_, T>>
    where
        T: 'static,
    {
        self.track();
        let guard = RUNTIME.list_signal.get(&self.id())?;
        Some(ListIter {
            guard,
            index: 0,
            _type: PhantomData,
        })
    }
}

/// 只读迭代器，持有 DashMap 读锁生命周期
pub struct ListIter<'a, T> {
    guard: Ref<'a, Id, Vec<ListItem>>,
    index: usize,
    _type: PhantomData<T>,
}

impl<'a, T: 'static> Iterator for ListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.guard.len() {
            return None;
        }
        let item = &self.guard[self.index];
        self.index += 1;
        Some(
            item.value
                .get_ref::<T>()
                .expect("iter: to downcast list item type fail"),
        )
    }
}
