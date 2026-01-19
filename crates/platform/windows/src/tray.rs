use crate::conversions::encode_wide::encode_wide;
use crate::window::get_instance_handle;
use crate::{Error, WindowId};
use flor_base::platform::{IconSource, MouseButton, TrayEvent, TrayManagerEntry, TrayOptions};
// 确保引入 IconSource
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use slotmap::{new_key_type, Key, KeyData, SlotMap};
use std::mem::size_of;
use std::sync::OnceLock;
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NIN_POPUPCLOSE, NIN_POPUPOPEN, NIN_SELECT, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIcon, CreateWindowExW, DefWindowProcW, LoadImageW, RegisterClassW, CW_USEDEFAULT, HICON,
    IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MBUTTONDBLCLK, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_USER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_OVERLAPPED,
};
use windows_core::PCWSTR;

// ============================================================================
// 全局状态
// ============================================================================

static ICON_HWND: OnceLock<WindowId> = OnceLock::new();
const WM_TRAY_CALLBACK: u32 = WM_USER + 233;

static TRAY_STORAGE: Lazy<TrayStorage> = Lazy::new(|| TrayStorage::default());

new_key_type! {
    pub struct TrayId;
}

impl flor_base::platform::TrayId for TrayId {}

struct TrayHIcon(pub HICON);
unsafe impl Send for TrayHIcon {}
unsafe impl Sync for TrayHIcon {}

#[derive(Default)]
pub struct TrayStorage {
    // 存储 HICON，依靠 RAII 自动释放
    icons: RwLock<SlotMap<TrayId, TrayHIcon>>,
    callback: RwLock<Option<Box<dyn Fn(TrayId, TrayEvent) + Send + Sync + 'static>>>,
}

fn icon_hwnd() -> HWND {
    ICON_HWND.get().expect("Tray not initialized").hwnd()
}

// ----------------------------------------------------------------------------
// ID 压缩算法
// ----------------------------------------------------------------------------

fn compress_id(id: TrayId) -> Result<u32, Error> {
    let raw = id.data().as_ffi();
    let idx = (raw & 0xFFFF_FFFF) as u32;
    let ver = (raw >> 32) as u32;

    if idx > 0xFFFF || ver > 0xFFFF {
        return Err(Error::from(E_FAIL));
    }
    Ok((ver << 16) | idx)
}

fn decompress_id(win_id: u32) -> TrayId {
    let idx = (win_id & 0xFFFF) as u64;
    let ver = (win_id >> 16) as u64;
    KeyData::from_ffi((ver << 32) | idx).into()
}

// ============================================================================
// 实现
// ============================================================================

pub struct Tray;

impl TrayManagerEntry for Tray {
    type TrayId = TrayId;
    type Error = Error;

    fn init() -> Result<(), Error> {
        Lazy::force(&TRAY_STORAGE);

        unsafe {
            let class_name_str = encode_wide("flor_tray_icon");
            let class_name = PCWSTR(class_name_str.as_ptr());

            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(tray_proc),
                lpszClassName: class_name,
                hInstance: get_instance_handle(),
                ..std::mem::zeroed()
            };
            RegisterClassW(&wnd_class);

            let hwnd = CreateWindowExW(
                WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
                class_name,
                PCWSTR::null(),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                0,
                CW_USEDEFAULT,
                0,
                None,
                None,
                Some(get_instance_handle()),
                Some(233i32 as _),
            )?;

            let _ = ICON_HWND.set(hwnd.into());
            Ok(())
        }
    }

    fn add(options: &TrayOptions) -> Result<Self::TrayId, Self::Error> {
        // 根据 IconSource 加载 HICON
        let h_icon = resolve_icon(&options.icon_path)?; // 注意：这里使用了你定义的字段名 icon_path

        // 1. 存入 SlotMap
        let tray_id = TRAY_STORAGE.icons.write().insert(TrayHIcon(h_icon));

        // 2. 压缩 ID
        let win_id = match compress_id(tray_id) {
            Ok(id) => id,
            Err(e) => {
                TRAY_STORAGE.icons.write().remove(tray_id); // RAII drop
                return Err(e);
            }
        };

        unsafe {
            let mut data = NOTIFYICONDATAW::default();
            data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            data.hWnd = icon_hwnd();
            data.uID = win_id;
            data.uFlags = NIF_MESSAGE;
            data.uCallbackMessage = WM_TRAY_CALLBACK;

            if !h_icon.is_invalid() {
                data.uFlags |= NIF_ICON;
                data.hIcon = h_icon;
            }

            if !options.tooltip.is_empty() {
                data.uFlags |= NIF_TIP;
                set_tooltip(&mut data.szTip, &options.tooltip);
            }

            if let Err(e) = Shell_NotifyIconW(NIM_ADD, &data).ok() {
                TRAY_STORAGE.icons.write().remove(tray_id); // RAII drop
                return Err(e);
            }

            Ok(tray_id)
        }
    }

    fn update(tray_id: Self::TrayId, options: &TrayOptions) -> Result<(), Self::Error> {
        if !TRAY_STORAGE.icons.read().contains_key(tray_id) {
            return Err(Error::from(E_INVALIDARG));
        }

        let win_id = compress_id(tray_id)?;

        unsafe {
            let mut data = NOTIFYICONDATAW::default();
            data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            data.hWnd = icon_hwnd();
            data.uID = win_id;

            // 只有当有新图标时才更新
            if let Some(source) = &options.icon_path {
                // 生成新句柄
                let h_new = resolve_icon_from_source(source)?;
                data.uFlags |= NIF_ICON;
                data.hIcon = h_new;

                // 替换旧句柄：SlotMap insert/get_mut 替换后，旧值被 Drop，触发 RAII 清理
                if let Some(h_old) = TRAY_STORAGE.icons.write().get_mut(tray_id) {
                    h_old.0 = h_new;
                }
            }

            if !options.tooltip.is_empty() {
                data.uFlags |= NIF_TIP;
                set_tooltip(&mut data.szTip, &options.tooltip);
            }

            if data.uFlags.0 != 0 {
                Shell_NotifyIconW(NIM_MODIFY, &data).ok()?;
            }
            Ok(())
        }
    }

    fn remove(tray_id: Self::TrayId) -> Result<(), Self::Error> {
        // 从 SlotMap 移除，触发 HICON 的 Drop (RAII)
        let _ = TRAY_STORAGE
            .icons
            .write()
            .remove(tray_id)
            .ok_or_else(|| Error::from(E_INVALIDARG))?;

        let win_id = compress_id(tray_id)?;

        unsafe {
            let mut data = NOTIFYICONDATAW::default();
            data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            data.hWnd = icon_hwnd();
            data.uID = win_id;

            Shell_NotifyIconW(NIM_DELETE, &data).ok()?;
            Ok(())
        }
    }

    fn on_callback(f: impl Fn(Self::TrayId, TrayEvent) + Send + Sync + 'static) {
        *TRAY_STORAGE.callback.write() = Some(Box::new(f));
    }
}

// ============================================================================
// 窗口过程
// ============================================================================
const NIN_KEYSELECT: u32 = WM_USER + 1;
unsafe extern "system" fn tray_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAY_CALLBACK => {
            let win_id = w_param.0 as u32;
            let tray_id = decompress_id(win_id);

            if TRAY_STORAGE.icons.read().contains_key(tray_id) {
                if let Some(cb) = TRAY_STORAGE.callback.read().as_ref() {
                    // l_param 的低位包含了具体的鼠标消息
                    let mouse_msg = l_param.0 as u32;

                    match mouse_msg {
                        // --- 按下 (MouseDown) ---
                        WM_LBUTTONDOWN => cb(tray_id, TrayEvent::MouseDown(MouseButton::Left)),
                        WM_RBUTTONDOWN => cb(tray_id, TrayEvent::MouseDown(MouseButton::Right)),
                        WM_MBUTTONDOWN => cb(tray_id, TrayEvent::MouseDown(MouseButton::Middle)),

                        // --- 松开 (MouseUp) ---
                        WM_LBUTTONUP => cb(tray_id, TrayEvent::MouseUp(MouseButton::Left)),
                        WM_RBUTTONUP => cb(tray_id, TrayEvent::MouseUp(MouseButton::Right)),
                        WM_MBUTTONUP => cb(tray_id, TrayEvent::MouseUp(MouseButton::Middle)),

                        // --- 双击 (MouseDoubleClick) ---
                        WM_LBUTTONDBLCLK => {
                            cb(tray_id, TrayEvent::MouseDoubleClick(MouseButton::Left))
                        }
                        WM_RBUTTONDBLCLK => {
                            cb(tray_id, TrayEvent::MouseDoubleClick(MouseButton::Right))
                        }
                        WM_MBUTTONDBLCLK => {
                            cb(tray_id, TrayEvent::MouseDoubleClick(MouseButton::Middle))
                        }

                        // --- 移动与悬停 ---
                        WM_MOUSEMOVE => cb(tray_id, TrayEvent::MouseMove),

                        // NIN_POPUPOPEN / CLOSE 需要 Shell 版本支持，
                        // 它们是实现 MouseEnter/Leave 最准确的 Windows 消息
                        NIN_POPUPOPEN => cb(tray_id, TrayEvent::MouseEnter),
                        NIN_POPUPCLOSE => cb(tray_id, TrayEvent::MouseLeave),

                        // 键盘选中映射为左键松开语义
                        NIN_SELECT | NIN_KEYSELECT => {
                            cb(tray_id, TrayEvent::MouseUp(MouseButton::Left))
                        }

                        _ => {}
                    }
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

// 统一处理 Option<IconSource>
fn resolve_icon(source_opt: &Option<IconSource>) -> Result<HICON, Error> {
    match source_opt {
        Some(source) => resolve_icon_from_source(source),
        None => Ok(HICON::default()),
    }
}

fn resolve_icon_from_source(source: &IconSource) -> Result<HICON, Error> {
    match source {
        IconSource::Path(path) => unsafe { load_hicon(path) },
        IconSource::Raw {
            width,
            height,
            rgba_data,
        } => unsafe { create_hicon_from_rgba(*width, *height, rgba_data) },
    }
}

unsafe fn load_hicon(path: &std::path::Path) -> Result<HICON, Error> {
    use std::os::windows::ffi::OsStrExt;
    let path_str = path.as_os_str();
    let wide_path: Vec<u16> = path_str.encode_wide().chain(Some(0)).collect();

    let handle = LoadImageW(
        None,
        PCWSTR(wide_path.as_ptr()),
        IMAGE_ICON,
        0,
        0,
        LR_LOADFROMFILE | LR_DEFAULTSIZE,
    )?;

    Ok(HICON(handle.0))
}

unsafe fn create_hicon_from_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<HICON, Error> {
    if rgba.len() != (width * height * 4) as usize {
        return Err(Error::from(E_INVALIDARG));
    }

    // RGBA -> BGRA 转换 (Windows GDI 默认 BGRA)
    let mut bgra = rgba.to_vec();
    for chunk in bgra.chunks_mut(4) {
        let tmp = chunk[0];
        chunk[0] = chunk[2]; // Swap Red
        chunk[2] = tmp; // Swap Blue
    }

    // 使用 CreateIcon 直接从内存创建
    let h_icon = CreateIcon(
        Some(get_instance_handle()),
        width as i32,
        height as i32,
        1,                // planes
        32,               // bits per pixel
        std::ptr::null(), // AND mask (null means implicit)
        bgra.as_ptr(),    // XOR bits
    )?;

    Ok(h_icon)
}

fn set_tooltip(target: &mut [u16; 128], text: &str) {
    let wide_text: Vec<u16> = text.encode_utf16().collect();
    let len = wide_text.len().min(127);
    target[..len].copy_from_slice(&wide_text[..len]);
    target[len] = 0;
}
