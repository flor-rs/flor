use crate::signal::effect::signal_effect::SignalEffect;
use crate::signal::id::{EffectId, Id};
use crate::signal::runtime::{RUNTIME, SCOPE};
use std::marker::PhantomData;

struct UpdaterEffect<C, U, T>
where
    T: 'static,
    C: Fn() -> T + 'static,
    U: Fn(T),
{
    id: Id,
    compute: C,
    on_change: U,
    _type: PhantomData<T>,
}

unsafe impl<C, U, T> Send for UpdaterEffect<C, U, T>
where
    T: 'static,
    C: Fn() -> T + 'static,
    U: Fn(T),
{
}

unsafe impl<C, U, T> Sync for UpdaterEffect<C, U, T>
where
    T: 'static,
    C: Fn() -> T + 'static,
    U: Fn(T),
{
}

impl<C, U, T> UpdaterEffect<C, U, T>
where
    T: 'static,
    C: Fn() -> T + 'static,
    U: Fn(T),
{
    fn init_effect(&self) -> T {
        let effect_id = self.id;
        // 备份旧标记
        let old_current_effect_id = SCOPE.get();
        // 写入新标记
        SCOPE.set(Some(effect_id));
        // 执行调用
        let new_value = (self.compute)();
        // 恢复旧标记
        SCOPE.set(old_current_effect_id);
        new_value
    }
}

impl<C, U, T> SignalEffect for UpdaterEffect<C, U, T>
where
    T: 'static,
    C: Fn() -> T + 'static,
    U: Fn(T),
{
    fn run_effect(&self) {
        let new_value = self.init_effect();
        (self.on_change)(new_value);
    }
}

pub fn create_updater<R>(compute: impl Fn() -> R + 'static, on_change: impl Fn(R) + 'static) -> R
where
    R: 'static,
{
    let effect_id = Id::next();
    let effect = UpdaterEffect {
        id: effect_id,
        compute,
        on_change,
        _type: PhantomData::default(),
    };
    let init_value = effect.init_effect();
    RUNTIME.effects.insert(effect_id, Box::new(effect));
    init_value
}

pub fn create_updater_with_id<R>(
    compute: impl Fn() -> R + 'static,
    on_change: impl Fn(R) + 'static,
) -> (EffectId, R)
where
    R: 'static,
{
    let effect_id = Id::next();
    let effect = UpdaterEffect {
        id: effect_id,
        compute,
        on_change,
        _type: PhantomData::default(),
    };
    let init_value = effect.init_effect();
    RUNTIME.effects.insert(effect_id, Box::new(effect));
    (effect_id, init_value)
}
