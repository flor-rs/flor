use flor_base::platform::MousePosition;

pub trait IntoMousePosition {
    fn into_mouse_position(self) -> MousePosition;
}

impl IntoMousePosition for isize {
    #[inline]
    fn into_mouse_position(self) -> MousePosition {
        // 原理：
        // 1. self as i16: 直接截断高位，拿到低16位 (X坐标)。
        // 2. (self >> 16) as i16: 右移后截断，拿到高16位 (Y坐标)。
        // 3. as i32: i16 转 i32 会自动进行符号扩展，保证负数坐标正确。
        MousePosition {
            x: self as i16 as i32,
            y: (self >> 16) as i16 as i32,
        }
    }
}