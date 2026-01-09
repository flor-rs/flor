use crate::conversions::key_state::IntoKeyState;
use crate::conversions::mouse_position::IntoMousePosition;
use crate::conversions::word::hiword_i16;
use crate::proc;
use flor_platform_base::{HandleResult, Message, ScrollAxis};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::WindowsAndMessaging::WHEEL_DELTA;

#[inline]
pub fn mouse_wheel(axis: ScrollAxis, hwnd: HWND, w_param: WPARAM, l_param: LPARAM) -> HandleResult {
    let wp = w_param.0 as u32;
    proc().window_proc(
        hwnd.into(),
        Message::MouseWheel {
            axis,
            delta: hiword_i16(wp) as f32 / WHEEL_DELTA as f32,
            key_state: MODIFIERKEYS_FLAGS(wp).into_key_state(),
            mouse_position: l_param.0.into_mouse_position(),
        },
    )
}
