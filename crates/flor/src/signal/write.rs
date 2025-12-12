use crate::signal::id::Id;
use crate::signal::runtime::RUNTIME;
use crate::signal::value::Value;

pub trait Write<T> {
    fn id(&self) -> Id;

    fn set(&self, new_value: T)
    where
        T: 'static,
    {
        RUNTIME.values.insert(self.id(), Value::new(new_value));
        RUNTIME.run_signal_effect(self.id());
    }
    fn update(&self, f: impl FnOnce(&mut T))
    where
        T: 'static,
    {
        if let Some(mut value) = RUNTIME.values.get_mut(&self.id()) {
            let x = value.value_mut();
            let t = x.get_mut_ref().expect("to downcast signal type fail");
            f(t);
        }
        // 触发所有订阅的变更
        RUNTIME.run_signal_effect(self.id());
    }

    fn destroy(&self) {
        RUNTIME.destroy_signal(self.id());
    }
}
