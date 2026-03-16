use crate::signal::{Id, Value, RUNTIME, SCOPE};
use std::any::Any;
use std::marker::PhantomData;

mod signal_effect;
mod updater_effect;

pub use {signal_effect::*, updater_effect::*};

struct Effect<T, F>
where
    T: 'static,
    F: Fn(Option<T>) -> T,
{
    id: Id,
    f: F,
    _type: PhantomData<T>,
}

unsafe impl<T, F> Sync for Effect<T, F> where F: Fn(Option<T>) -> T {}

unsafe impl<T, F> Send for Effect<T, F> where F: Fn(Option<T>) -> T {}

impl<T, F> SignalEffect for Effect<T, F>
where
    T: 'static,
    F: Fn(Option<T>) -> T,
{
    fn run_effect(&self) {
        self.init_effect();
    }
}

impl<T, F> Effect<T, F>
where
    T: 'static,
    F: Fn(Option<T>) -> T,
{
    fn init_effect(&self) {
        let effect_id = self.id;
        // 备份旧标记
        let old_current_effect_id = SCOPE.get();
        // 写入新标记
        SCOPE.set(Some(effect_id));
        // 执行调用
        let last_value = RUNTIME
            .values
            .remove(&self.id)
            .and_then(|(_id, value)| value.into_inner().ok());
        let current_value = (self.f)(last_value); // 1
        RUNTIME.values.insert(self.id, Value::new(current_value));
        // 恢复旧标记
        SCOPE.set(old_current_effect_id);
    }
}

pub fn create_effect<T>(f: impl Fn(Option<T>) -> T + 'static)
where
    T: Any + 'static,
{
    let id = Id::next_effect_id();
    let effect = Effect {
        id,
        f,
        _type: Default::default(),
    };
    effect.run_effect();
    RUNTIME.effects.insert(id, Box::new(effect));
}
