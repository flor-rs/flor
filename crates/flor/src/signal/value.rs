use std::any::Any;

mod v_read;
mod v_read_signal;
mod v_rw_signal;
mod v_signal_ref;
mod v_write;
mod v_write_signal;

pub use {
    v_read::*, v_read_signal::*, v_rw_signal::*, v_signal_ref::*, v_write::*, v_write_signal::*,
    v_write_signal::*,
};

#[derive(Debug)]
pub struct Value(Box<dyn Any>);
unsafe impl Send for Value {}
unsafe impl Sync for Value {}

impl Value {
    #[inline]
    pub fn new<T: 'static>(value: T) -> Self {
        Self(Box::new(value))
    }

    pub fn get<T: Clone + 'static>(&self) -> Option<T> {
        self.0.downcast_ref::<T>().cloned()
    }

    // 获取不可变引用
    pub fn get_ref<T: 'static>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }

    // 获取可变引用
    pub fn get_mut_ref<T: 'static>(&mut self) -> Option<&mut T> {
        self.0.downcast_mut::<T>()
    }

    pub fn into_inner<T: 'static>(self) -> Result<T, Box<dyn Any>> {
        self.0.downcast::<T>().map(|b| *b)
    }
}
