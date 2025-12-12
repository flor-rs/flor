use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};
use flor_platform_base::MousePosition;

pub trait IntoMousePosition {
    fn into_mouse_position(self) -> MousePosition;
}

impl IntoMousePosition for isize {
    #[inline]
    fn into_mouse_position(self) -> MousePosition {
        let x = GET_X_LPARAM(self) as i32;
        let y = GET_Y_LPARAM(self) as i32;
        MousePosition { x, y }
    }
}
