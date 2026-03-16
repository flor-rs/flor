use crate::signal::{Signal, Value, RUNTIME};

pub trait Write<T>: Signal {
    fn set(&self, new_value: T)
    where
        T: 'static,
    {
        let mut value = RUNTIME.values.get_mut(&self.id()).expect(
            "invalid signal id: this signal has likely been destroyed.\n\
                    If the signal lifetime is not guaranteed, use `try_set()` instead of `set()`.",
        );
        *value = Value::new(new_value);
        RUNTIME.run_signal_effect(self.id());
    }

    fn try_set(&self, new_value: T) -> bool
    where
        T: 'static,
    {
        if let Some(mut value) = RUNTIME.values.get_mut(&self.id()) {
            *value = Value::new(new_value);
            RUNTIME.run_signal_effect(self.id());
            true
        } else {
            false
        }
    }

    fn update(&self, f: impl FnOnce(&mut T))
    where
        T: 'static,
    {
        let mut value = RUNTIME
            .values
            .get_mut(&self.id())
            .expect(
                "invalid signal id: this signal has likely been destroyed.\n\
                    If the signal lifetime is not guaranteed, use `try_update()` instead of `update()`.",
            );
        let x = value.value_mut();
        let t = x.get_mut_ref().expect("to downcast signal type fail");
        f(t);
        // 触发所有订阅的变更
        RUNTIME.run_signal_effect(self.id());
    }

    fn try_update(&self, f: impl FnOnce(&mut T)) -> bool
    where
        T: 'static,
    {
        if let Some(mut value) = RUNTIME.values.get_mut(&self.id()) {
            let x = value.value_mut();
            let t = x
                .get_mut_ref()
                .expect("try_update: downcast signal type fail");
            f(t);
            RUNTIME.run_signal_effect(self.id());
            true
        } else {
            false
        }
    }
}
