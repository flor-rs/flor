// ============================================================================
// Prop Traits 生成宏 - 支持固定值或闭包
// ============================================================================

/// 定义属性 Prop trait 的宏
/// 自动生成 trait 定义、固定值实现和闭包实现
///
/// # 用法
///
/// ```rust
/// // Copy 类型 (bool, f32, enum 等)
/// define_prop!(copy BoolProp, bool);
///
/// // Clone 类型 (String, Color 等)
/// define_prop!(clone ColorProp, Color);
///
/// // 带额外类型转换
/// define_prop!(copy F32Prop, f32, extra: i32 => |v| v as f32);
/// define_prop!(clone StringProp, String, extra: &'static str => |s: &str| s.to_string());
/// ```
#[macro_export]
macro_rules! define_prop {
    // Copy 类型版本 (bool, f32, i32, enum 等)
    (copy $trait_name:ident, $type:ty) => {
        pub trait $trait_name: 'static {
            fn make(&self) -> $type;
        }

        impl $trait_name for $type {
            fn make(&self) -> $type {
                *self
            }
        }

        impl<F> $trait_name for F
        where
            F: Fn() -> $type + 'static,
        {
            fn make(&self) -> $type {
                (self)()
            }
        }
    };
    
    // Clone 类型版本 (String, Color 等)
    (clone $trait_name:ident, $type:ty) => {
        pub trait $trait_name: 'static {
            fn make(&self) -> $type;
        }

        impl $trait_name for $type {
            fn make(&self) -> $type {
                self.clone()
            }
        }

        impl<F> $trait_name for F
        where
            F: Fn() -> $type + 'static,
        {
            fn make(&self) -> $type {
                (self)()
            }
        }
    };
    
    // Copy 类型带额外 impl 的版本
    (copy $trait_name:ident, $type:ty, extra: $($extra_type:ty => $convert:expr),* $(,)?) => {
        pub trait $trait_name: 'static {
            fn make(&self) -> $type;
        }

        impl $trait_name for $type {
            fn make(&self) -> $type {
                *self
            }
        }

        $(
            impl $trait_name for $extra_type {
                fn make(&self) -> $type {
                    $convert(*self)
                }
            }
        )*

        impl<F> $trait_name for F
        where
            F: Fn() -> $type + 'static,
        {
            fn make(&self) -> $type {
                (self)()
            }
        }
    };
    
    // Clone 类型带额外 impl
    (clone $trait_name:ident, $type:ty, extra: $($extra_type:ty => $convert:expr),* $(,)?) => {
        pub trait $trait_name: 'static {
            fn make(&self) -> $type;
        }

        impl $trait_name for $type {
            fn make(&self) -> $type {
                self.clone()
            }
        }

        $(
            impl $trait_name for $extra_type {
                fn make(&self) -> $type {
                    $convert(self)
                }
            }
        )*

        impl<F> $trait_name for F
        where
            F: Fn() -> $type + 'static,
        {
            fn make(&self) -> $type {
                (self)()
            }
        }
    };
}

// ============================================================================
// 预定义的 std 类型 Prop Traits
// ============================================================================

// 布尔类型
define_prop!(copy BoolProp, bool);

// 有符号整数
define_prop!(copy I8Prop, i8);
define_prop!(copy I16Prop, i16);
define_prop!(copy I32Prop, i32);
define_prop!(copy I64Prop, i64);
define_prop!(copy I128Prop, i128);
define_prop!(copy IsizeProp, isize);

// 无符号整数
define_prop!(copy U8Prop, u8);
define_prop!(copy U16Prop, u16);
define_prop!(copy U32Prop, u32);
define_prop!(copy U64Prop, u64);
define_prop!(copy U128Prop, u128);
define_prop!(copy UsizeProp, usize);

// 浮点数
define_prop!(copy F32Prop, f32);
define_prop!(copy F64Prop, f64);

// 字符
define_prop!(copy CharProp, char);

// 字符串
define_prop!(clone StringProp, String, extra: &'static str => |s: &str| s.to_string());
