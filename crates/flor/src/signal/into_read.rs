use crate::signal::{ConstSignal, Read, ReadSignal, RwSignal};

pub trait IntoRead<T> {
    type Reader: Read<T>;
    fn into_read(self) -> Self::Reader;
}

// 使用宏来覆盖你常用的所有基础类型
#[macro_export]
macro_rules! impl_into_read {
    ($($t:ty),*) => {
        $(
            impl IntoRead<$t> for $t {
                type Reader = ConstSignal<$t>;
                fn into_read(self) -> Self::Reader {
                    ConstSignal::new(self)
                }
            }
        )*
    };
}

impl_into_read!(
    (),
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    bool,
    char,
    String
);

impl<T: Clone> IntoRead<T> for ConstSignal<T> {
    type Reader = Self;
    fn into_read(self) -> Self {
        self
    }
}

impl<T: Clone> IntoRead<T> for ReadSignal<T> {
    type Reader = Self;
    fn into_read(self) -> Self {
        self
    }
}

impl<T: Clone> IntoRead<T> for RwSignal<T> {
    type Reader = Self;
    fn into_read(self) -> Self {
        self
    }
}

impl IntoRead<String> for &str {
    type Reader = ConstSignal<String>;
    fn into_read(self) -> Self::Reader {
        ConstSignal::new(self.to_string())
    }
}

impl IntoRead<String> for &String {
    type Reader = ConstSignal<String>;
    fn into_read(self) -> Self::Reader {
        ConstSignal::new(self.clone())
    }
}
