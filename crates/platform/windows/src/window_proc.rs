use log::{debug, info, trace};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPSTR,
    GCS_RESULTSTR, HIMC, IME_COMPOSITION_STRING,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_CONTROL, VK_MENU, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::conversions::key_code::FromVkCode;
use crate::conversions::key_state::IntoKeyState;
use crate::conversions::mouse_position::IntoMousePosition;
use crate::proc_handler::proc;
use crate::util::{hiword, loword};
use flor_platform_base::{HandleResult, InputEvent};
use flor_platform_base::{KeyCode, Message};

pub(crate) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    info!("window proc {:?}", (hwnd, msg, wparam, lparam));
    // if let Some(proc_handler) = USER_PROC_HANDLER {
    //     if let HandlerResult::Handled(result) = proc_handler(hwnd, msg, wparam, lparam) {
    //         debug!("user proc handler {:?}",result);
    //         return result;
    //     } else {
    //         trace!("user proc handler not handled. continue process.");
    //     }
    // }
    let handle_result = match msg {
        WM_PAINT => {
            debug!("WM_PAINT");
            let mut ps = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut ps);
            // todo 两次绘制，第二次绘制加入裁剪，试试能不能让分层窗口支持传统组件。比如webview
            let r = proc().window_proc(hwnd.into(), Message::Draw);
            let _ = EndPaint(hwnd, &mut ps);
            trace!("事件已通过自定义处理完成");
            r
        }
        WM_SIZE => {
            debug!("WM_SIZE");
            let width = loword(lparam.0 as u32);
            let height = hiword(lparam.0 as u32);
            proc().window_proc(
                hwnd.into(),
                Message::Resize {
                    width: width as u32,
                    height: height as u32,
                },
            )
        }
        WM_IME_STARTCOMPOSITION => proc().window_proc(hwnd.into(), Message::ImeStart),
        WM_IME_COMPOSITION => {
            let mut handle_result = HandleResult::Default;
            let h_imc = unsafe { ImmGetContext(hwnd) };
            // 为空就走默认
            if !h_imc.is_invalid() {
                if (lparam.0 & GCS_RESULTSTR.0 as isize) != 0 {
                    let text = get_composition_string(h_imc, GCS_RESULTSTR);
                    handle_result = proc()
                        .window_proc(hwnd.into(), Message::ImeInput(InputEvent::ImeEnd(text)));
                }
                if (lparam.0 & GCS_COMPSTR.0 as isize) != 0 {
                    let preedit = get_composition_string(h_imc, GCS_COMPSTR);
                    handle_result = proc()
                        .window_proc(hwnd.into(), Message::ImeInput(InputEvent::ImeIng(preedit)));
                }
                unsafe {
                    // 这里有异常返回，如何处理好？这里没法 ? 因为是回调函数
                    let _ = ImmReleaseContext(hwnd, h_imc);
                };
            }
            handle_result
        }
        WM_IME_ENDCOMPOSITION => proc().window_proc(hwnd.into(), Message::ImeEnd),
        WM_CHAR => {
            // 1. 从 wparam 获取 UTF-32 字符代码
            let char_code = wparam.0 as u32;
            // 2. 转换为 Rust char
            if let Some(ch) = std::char::from_u32(char_code) {
                // 3. 区分普通字符和控制字符
                // Windows 下 \r (回车) 和 \t (Tab) 也是 Control，
                // 但通常文本框需要处理它们，所以视情况归类。
                let input_state = if ch.is_control() {
                    // 特殊处理：保留回车(\r) 和 Tab(\t) 作为普通输入，
                    // 其他的 (如 Backspace \x08, Esc \x1b) 归为 Control
                    match ch {
                        '\r' | '\t' => InputEvent::Char(ch),
                        _ => InputEvent::Control(ch),
                    }
                } else {
                    InputEvent::Char(ch)
                };

                // 4. 发送消息
                // 注意：这里没有 String::new()，完全在栈上操作，性能最高
                proc().window_proc(hwnd.into(), Message::ImeInput(input_state))
            } else {
                HandleResult::Default
            }
        }
        WM_DPICHANGED => {
            debug!("WM_DPICHANGED");
            // 1. 分别提取 X 和 Y 的 DPI
            let dpi_x = (wparam.0 & 0xFFFF) as f32;
            let dpi_y = ((wparam.0 >> 16) & 0xFFFF) as f32;

            proc().window_proc(hwnd.into(), Message::DpiChange { dpi_x, dpi_y })
        }
        WM_LBUTTONDBLCLK => proc().window_proc(
            hwnd.into(),
            Message::LButtonDoubleClick {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_LBUTTONDOWN => proc().window_proc(
            hwnd.into(),
            Message::LButtonDown {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_LBUTTONUP => proc().window_proc(
            hwnd.into(),
            Message::LButtonUp {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_RBUTTONDBLCLK => proc().window_proc(
            hwnd.into(),
            Message::RButtonDoubleClick {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_RBUTTONDOWN => proc().window_proc(
            hwnd.into(),
            Message::RButtonDown {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_RBUTTONUP => proc().window_proc(
            hwnd.into(),
            Message::RButtonUp {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_MBUTTONDBLCLK => proc().window_proc(
            hwnd.into(),
            Message::MButtonDoubleClick {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_MBUTTONDOWN => proc().window_proc(
            hwnd.into(),
            Message::MButtonDown {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_MBUTTONUP => proc().window_proc(
            hwnd.into(),
            Message::MButtonUp {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        // todo x系列按钮暂时不写
        WM_MOUSEMOVE => proc().window_proc(
            hwnd.into(),
            Message::MouseMove {
                key_state: MODIFIERKEYS_FLAGS(wparam.0 as u32).into_key_state(),
                mouse_position: lparam.0.into_mouse_position(),
            },
        ),
        WM_NCMOUSELEAVE => {
            info!("WM_NCMOUSELEAVE");
            proc().window_proc(hwnd.into(), Message::MouseLeave)
        }
        WM_DESTROY => {
            debug!("WM_DESTROY",);
            let r = proc().window_proc(hwnd.into(), Message::WindowDestroy);
            trace!("WM_DESTROY execute end.");
            r
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
            HandleResult::Default
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
                Message::KeyDown {
                    code,
                    is_alt,
                    is_ctrl,
                    is_shift,
                }
            } else {
                Message::KeyUp {
                    code,
                    is_alt,
                    is_ctrl,
                    is_shift,
                }
            };

            proc().window_proc(hwnd.into(), msg)
        }

        _ => HandleResult::Default,
    };
    if let HandleResult::Handled = handle_result {
        return LRESULT(0);
    }

    let def_result = DefWindowProcW(hwnd, msg, wparam, lparam);
    trace!("事件通过默认处理器结果： {:?}", def_result);
    def_result
}

fn get_composition_string(h_imc: HIMC, flag: IME_COMPOSITION_STRING) -> String {
    // 第一次调用传 0，获取需要的字节长度
    let len = unsafe { ImmGetCompositionStringW(h_imc, flag, None, 0) };
    if len <= 0 {
        return String::new();
    }

    let mut buf = vec![0u16; (len / 2) as usize];
    // 第二次调用，真正把数据拷出来
    unsafe { ImmGetCompositionStringW(h_imc, flag, Some(buf.as_mut_ptr() as _), len as u32) };

    String::from_utf16_lossy(&buf)
}
