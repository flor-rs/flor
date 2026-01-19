use crate::conversions::key_state::IntoKeyState;
use crate::conversions::mouse_position::IntoMousePosition;
use crate::conversions::word::hiword_i16;
use crate::proc;
use flor_base::platform::{HandleResult, Message, MousePosition, ScrollAxis};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::WindowsAndMessaging::WHEEL_DELTA;

#[inline]
pub fn mouse_wheel(axis: ScrollAxis, hwnd: HWND, w_param: WPARAM, l_param: LPARAM) -> HandleResult {
    let wp = w_param.0 as u32;

    // WM_MOUSEWHEEL 的 LPARAM 包含的是屏幕坐标，需要转换为客户区坐标
    let screen_pos = l_param.0.into_mouse_position();
    let mut pt = POINT {
        x: screen_pos.x,
        y: screen_pos.y,
    };

    // 转换屏幕坐标为客户区坐标
    unsafe {
        let _ = ScreenToClient(hwnd, &mut pt);
    }

    let client_pos = MousePosition { x: pt.x, y: pt.y };

    proc().window_proc(
        hwnd.into(),
        Message::MouseWheel {
            axis,
            delta: hiword_i16(wp) as f32 / WHEEL_DELTA as f32,
            key_state: MODIFIERKEYS_FLAGS(wp).into_key_state(),
            mouse_position: client_pos,
        },
    )
}
