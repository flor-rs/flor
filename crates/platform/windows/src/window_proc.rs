mod mouse_wheel;

use log::{debug, info, trace};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPSTR, GCS_RESULTSTR, HIMC,
    IME_COMPOSITION_STRING,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_CONTROL, VK_MENU, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::conversions::key_code::FromVkCode;
use crate::conversions::key_state::IntoKeyState;
use crate::conversions::mouse_position::IntoMousePosition;
use crate::conversions::word::{hiword_u16, loword_u16};
use crate::proc_handler::proc;
use crate::window_proc::mouse_wheel::mouse_wheel;
use flor_base::platform::{HandleResult, InputEvent, ScrollAxis};
use flor_base::platform::{KeyCode, Message};

pub(crate) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    debug!("window proc {:?}", (hwnd, msg, w_param, l_param));
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
            let width = loword_u16(l_param.0 as u32);
            let height = hiword_u16(l_param.0 as u32);
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
                if (l_param.0 & GCS_RESULTSTR.0 as isize) != 0 {
                    let text = get_composition_string(h_imc, GCS_RESULTSTR);
                    handle_result = proc()
                        .window_proc(hwnd.into(), Message::ImeInput(InputEvent::ImeEnd(text)));
                }
                if (l_param.0 & GCS_COMPSTR.0 as isize) != 0 {
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
            let char_code = w_param.0 as u32;
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
            let dpi_x = (w_param.0 & 0xFFFF) as f32;
            let dpi_y = ((w_param.0 >> 16) & 0xFFFF) as f32;

            proc().window_proc(hwnd.into(), Message::DpiChange { dpi_x, dpi_y })
        }
        WM_SETCURSOR => proc().window_proc(hwnd.into(), Message::Cursor),
        WM_LBUTTONDBLCLK => proc().window_proc(
            hwnd.into(),
            Message::LButtonDoubleClick {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        WM_LBUTTONDOWN => proc().window_proc(
            hwnd.into(),
            Message::LButtonDown {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        WM_LBUTTONUP => proc().window_proc(
            hwnd.into(),
            Message::LButtonUp {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        WM_RBUTTONDBLCLK => proc().window_proc(
            hwnd.into(),
            Message::RButtonDoubleClick {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        WM_RBUTTONDOWN => proc().window_proc(
            hwnd.into(),
            Message::RButtonDown {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        WM_RBUTTONUP => proc().window_proc(
            hwnd.into(),
            Message::RButtonUp {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        WM_MBUTTONDBLCLK => proc().window_proc(
            hwnd.into(),
            Message::MButtonDoubleClick {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        WM_MBUTTONDOWN => proc().window_proc(
            hwnd.into(),
            Message::MButtonDown {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        WM_MBUTTONUP => proc().window_proc(
            hwnd.into(),
            Message::MButtonUp {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
            },
        ),
        // todo x系列按钮暂时不写
        WM_MOUSEMOVE => proc().window_proc(
            hwnd.into(),
            Message::MouseMove {
                key_state: MODIFIERKEYS_FLAGS(w_param.0 as u32).into_key_state(),
                mouse_position: l_param.0.into_mouse_position(),
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
            let mut prevent = false;
            proc().window_proc(
                hwnd.into(),
                Message::CloseRequested {
                    prevent: &mut prevent,
                },
            );
            trace!("HandleResult::CloseRequested({prevent})");
            if prevent {
                HandleResult::Default
            } else {
                HandleResult::Handled
            }
        }
        WM_KEYDOWN | WM_KEYUP => {
            let is_down = msg == WM_KEYDOWN;
            let vk = VIRTUAL_KEY(w_param.0 as u16);

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
        WM_SETTINGCHANGE => {
            let spi_action = w_param.0 as u32;
            #[cfg(feature = "theme-change")]
            if is_lparam_str(l_param, "ImmersiveColorSet") {
                // 分发事件，但忽略返回值（或者确保你的 Message::ThemeChanged 也是返回 Default）
                // 这里假设我们只是为了通知框架层去更新状态，而不阻断系统消息
                let _ = proc().window_proc(hwnd.into(), Message::ThemeChanged(get_current_theme()));
            }

            // 2. 检测工作区变化 (任务栏移动/分辨率)
            // 注意：这里用 independent 'if' 比 'else if' 更稳健，虽然通常 wparam 和 lparam 不会同时有效
            if SYSTEM_PARAMETERS_INFO_ACTION(spi_action) == SPI_SETWORKAREA {
                let _ = proc().window_proc(hwnd.into(), Message::WorkAreaChanged);
            }

            // 3. 检测滚轮设置
            if SYSTEM_PARAMETERS_INFO_ACTION(spi_action) == SPI_SETWHEELSCROLLLINES {
                let mut lines: u32 = 3;
                // 主动查询最新的设置值
                unsafe {
                    let _ = SystemParametersInfoW(
                        SPI_GETWHEELSCROLLLINES,
                        0,
                        Some(&mut lines as *mut _ as *mut std::ffi::c_void),
                        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                    );
                }
                let _ = proc().window_proc(hwnd.into(), Message::WheelSettingsChanged(lines));
            }

            // 【关键】WM_SETTINGCHANGE 建议始终交给 DefWindowProc 处理后续
            // 即使我们处理了，系统可能还有其他组件关心这个变化
            HandleResult::Default
        }
        WM_MOUSEWHEEL => mouse_wheel(ScrollAxis::Vertical, hwnd.into(), w_param, l_param),
        WM_MOUSEHWHEEL => mouse_wheel(ScrollAxis::Horizontal, hwnd.into(), w_param, l_param),
        _ => HandleResult::Default,
    };
    if let HandleResult::Handled = handle_result {
        return LRESULT(0);
    }

    let def_result = DefWindowProcW(hwnd, msg, w_param, l_param);
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

#[cfg(feature = "theme-change")]
unsafe fn is_lparam_str(lparam: LPARAM, target: &str) -> bool {
    let ptr = lparam.0 as *const u16;
    if ptr.is_null() {
        return false;
    }

    // 将 Rust str 转换为 utf-16 迭代器进行逐个比较
    // 这种方式避免了分配 Vec<u16>，性能最高
    let mut target_iter = target.encode_utf16();
    let mut ptr_offset = 0;

    while let Some(target_char) = target_iter.next() {
        let mem_char = *ptr.add(ptr_offset);
        if mem_char != target_char {
            return false;
        }
        ptr_offset += 1;
    }

    // 检查 C 字符串结尾是否为 \0，确保不是前缀匹配
    *ptr.add(ptr_offset) == 0
}

/// 查询当前注册表，判断是深色还是浅色
#[cfg(feature = "theme-change")]
fn get_current_theme() -> flor_base::platform::ThemeMode {
    unsafe {
        let mut value: u32 = 0;
        let mut size = size_of::<u32>() as u32;
        // 微软标准的深色模式注册表路径
        let sub_key =
            windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let value_name = windows::core::w!("AppsUseLightTheme");

        let result = windows::Win32::System::Registry::RegGetValueW(
            windows::Win32::System::Registry::HKEY_CURRENT_USER,
            sub_key,
            value_name,
            windows::Win32::System::Registry::RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut _ as *mut _),
            Some(&mut size),
        );

        // 0 = Dark, 1 = Light. 读取失败默认 Light
        if result.is_ok() && value == 0 {
            flor_base::platform::ThemeMode::Dark
        } else {
            flor_base::platform::ThemeMode::Light
        }
    }
}
