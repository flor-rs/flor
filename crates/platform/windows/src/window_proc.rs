use log::{debug, trace};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VIRTUAL_KEY, VK_1, VK_CONTROL, VK_LSHIFT, VK_MENU, VK_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::*;

use flor_platform_base::{KeyCode, Message};
use flor_platform_base::HandleResult;
use crate::conversions::key_code::FromVkCode;
use crate::conversions::key_state::IntoKeyState;
use crate::conversions::mouse_position::IntoMousePosition;
use crate::proc_handler::proc;
use crate::util::{hiword, loword};

pub(crate) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // trace!("window proc {:?}",(hwnd, msg, wparam, lparam));
    // if let Some(proc_handler) = USER_PROC_HANDLER {
    //     if let HandlerResult::Handled(result) = proc_handler(hwnd, msg, wparam, lparam) {
    //         debug!("user proc handler {:?}",result);
    //         return result;
    //     } else {
    //         trace!("user proc handler not handled. continue process.");
    //     }
    // }
    match msg {
        WM_CREATE => {}
        WM_PAINT => {
            debug!("WM_PAINT");
            // todo 两次绘制，第二次绘制加入裁剪，试试能不能让分层窗口支持传统组件。比如webview
            let _ = proc().window_proc(hwnd.into(), Message::Draw);
            trace!("事件已通过自定义处理完成");
        }
        WM_SIZE => {
            debug!("WM_SIZE");
            let width = loword(lparam.0 as u32);
            let height = hiword(lparam.0 as u32);
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::Resize {
                    width: width as u32,
                    height: height as u32,
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_LBUTTONDBLCLK => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::LButtonDoubleClick {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_LBUTTONDOWN => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::LButtonDown {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_LBUTTONUP => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::LButtonUp {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_RBUTTONDBLCLK => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::RButtonDoubleClick {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_RBUTTONDOWN => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::RButtonDown {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_RBUTTONUP => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::RButtonUp {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_MBUTTONDBLCLK => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::MButtonDoubleClick {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_MBUTTONDOWN => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::MButtonDown {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        WM_MBUTTONUP => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::MButtonUp {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }
        // todo x系列按钮暂时不写
        WM_MOUSEMOVE => {
            if let HandleResult::Default = proc().window_proc(
                hwnd.into(),
                Message::MouseMove {
                    key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                    mouse_position: lparam.0.into_mouse_position(),
                },
            ) {
                return LRESULT(0);
            }
        }

        WM_DESTROY => {
            debug!("WM_DESTROY",);
            let _ = proc().window_proc(hwnd.into(), Message::WindowDestroy);
            trace!("WM_DESTROY execute end.");
        }
        WM_CLOSE => {
            debug!("WM_CLOSE");
            if let HandleResult::WindowClose(is_close) =
                proc().window_proc(hwnd.into(), Message::Close)
            {
                trace!("HandleResult::Close({is_close})");
                if !is_close {
                    return LRESULT(0);
                }
            }
        }
        WM_KEYDOWN | WM_KEYUP => {
            let is_down = msg == WM_KEYDOWN;
            let vk = VIRTUAL_KEY(wparam.0 as u16);

            // 修饰键状态
            let (is_alt, is_ctrl, is_shift) = unsafe {
                (
                    GetKeyState(VK_MENU.0 as i32) as u16 & 0x8000 != 0,
                    GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000 != 0,
                    GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000 != 0,
                )
            };

            let code = KeyCode::from_vk(vk);
            let msg = if is_down {
                Message::KeyDown { code, is_alt, is_ctrl, is_shift }
            } else {
                Message::KeyUp { code, is_alt, is_ctrl, is_shift }
            };

            let _ = proc().window_proc(hwnd.into(), msg);
        }

        _ => {}
    }
    let def_result = DefWindowProcW(hwnd, msg, wparam, lparam);
    trace!("事件通过默认处理器结果： {:?}", def_result);
    def_result
}
