use crate::conversions::encode_wide::encode_wide;
use crate::cursor::Cursor;
use crate::window_id::WindowId;
use flor_base::platform::{CursorHandle, WindowApi, WindowMode, WindowOperations};
use once_cell::sync::Lazy;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{ClientToScreen, InvalidateRect, UpdateWindow};
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;
use windows::Win32::UI::Input::Ime::{
    ImmAssociateContext, ImmGetContext, ImmReleaseContext, ImmSetCompositionWindow,
    ImmSetOpenStatus, CFS_POINT, COMPOSITIONFORM, HIMC,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, GetClientRect, GetWindowPlacement, GetWindowRect, LoadCursorW,
    RegisterClassExW, SendMessageW, SetCursor, SetWindowPos, ShowWindow, CS_DBLCLKS, CS_HREDRAW,
    CS_VREDRAW, CW_USEDEFAULT, HMENU, HTCAPTION, IDC_ARROW, SC_MOVE, SHOW_WINDOW_CMD,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE,
    SW_RESTORE, SW_SHOW, SW_SHOWMAXIMIZED, SW_SHOWMINIMIZED, WINDOWPLACEMENT, WINDOW_EX_STYLE,
    WM_SYSCOMMAND, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};
use windows_core::{Error, PCWSTR};

// taken from winit's code base
// https://github.com/rust-windowing/winit/blob/ee88e38f13fbc86a7aafae1d17ad3cd4a1e761df/src/platform_impl/windows/util.rs#L138
pub fn get_instance_handle() -> HINSTANCE {
    // Gets the instance handle by taking the address of the
    // pseudo-variable created by the microsoft linker:
    // https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483

    // This is preferred over GetModuleHandle(NULL) because it also works in DLLs:
    // https://stackoverflow.com/questions/21718027/getmodulehandlenull-vs-hinstance
    extern "C" {
        static __ImageBase: IMAGE_DOS_HEADER;
    }

    unsafe { HINSTANCE(&__ImageBase as *const _ as _) }
}

static CLASS_NAME: Lazy<Vec<u16>> = Lazy::new(|| {
    let class_name = encode_wide("flor_window");
    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
        lpfnWndProc: Some(crate::window_proc::window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: get_instance_handle(),
        hIcon: Default::default(),
        hCursor: unsafe { LoadCursorW(Some(HINSTANCE::default()), IDC_ARROW).unwrap_or_default() },
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        hIconSm: Default::default(),
    };
    unsafe {
        RegisterClassExW(&wc);
    }
    class_name
});

impl WindowApi for WindowId {
    type Error = Error;
    type Cursor = Cursor;

    fn create_window(title: &str, width: u32, height: u32) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(CLASS_NAME.as_ptr()),
                PCWSTR(encode_wide(title).as_ptr()),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                Some(HWND::default()),
                Some(HMENU::default()),
                Some(HINSTANCE::default()),
                None,
            )?;
            Ok(hwnd.into())
        }
    }

    fn update_window(&self) -> Result<(), Self::Error> {
        unsafe {
            UpdateWindow(self.hwnd()).ok()?;
            Ok(())
        }
    }

    fn show(&self) -> Result<(), Self::Error> {
        unsafe {
            let _ = ShowWindow(self.hwnd(), SW_SHOW);
            Ok(())
        }
    }

    fn hide(&self) -> Result<(), Self::Error> {
        unsafe {
            let _ = ShowWindow(self.hwnd(), SW_HIDE);
            Ok(())
        }
    }

    fn set_window_mode(&self, mode: WindowMode) -> Result<(), Self::Error> {
        let cmd = match mode {
            WindowMode::Normal => SW_RESTORE,
            WindowMode::Minimized => SW_MINIMIZE,
            WindowMode::Maximized => SW_MAXIMIZE,
            // 简单映射：全屏暂时等同于最大化 (如需真全屏需修改 Style)
            WindowMode::Fullscreen => SW_MAXIMIZE,
        };
        unsafe {
            let _ = ShowWindow(self.hwnd(), cmd);
            Ok(())
        }
    }

    fn get_window_mode(&self) -> Result<WindowMode, Self::Error> {
        unsafe {
            let mut placement = WINDOWPLACEMENT {
                length: size_of::<WINDOWPLACEMENT>() as u32,
                ..Default::default()
            };
            GetWindowPlacement(self.hwnd(), &mut placement)?;

            match SHOW_WINDOW_CMD(placement.showCmd as i32) {
                x if x == SW_SHOWMINIMIZED => Ok(WindowMode::Minimized),
                x if x == SW_SHOWMAXIMIZED => Ok(WindowMode::Maximized),
                _ => Ok(WindowMode::Normal),
            }
        }
    }

    fn get_scale_factor(&self) -> Result<f32, Self::Error> {
        // unsafe {
        //     // 获取窗口特定的 DPI (Windows 10 1607+)
        //     let dpi = GetDpiForWindow(self.hwnd());
        //     // 标准 DPI 是 96
        //     Ok(dpi as f32 / 96.0)
        // }
        todo!()
    }

    fn get_dpi(&self) -> Result<(f32, f32), Self::Error> {
        #[cfg(not(feature = "monitor"))]
        unsafe {
            #[cfg(feature = "win7-compat")]
            {
                let dpi = crate::win7_compat::get_dpi_compatible(self.hwnd()) as f32;
                Ok((dpi, dpi))
            }
            #[cfg(not(feature = "win7-compat"))]
            {
                use windows::Win32::UI::HiDpi::GetDpiForWindow;
                let dpi = GetDpiForWindow(self.hwnd());
                let val = if dpi == 0 { 96.0 } else { dpi as f32 };
                Ok((val, val))
            }
        }
        #[cfg(feature = "monitor")]
        {
            use crate::base::MonitorApi;
            use crate::Monitor;
            // 原理是 MonitorFromWindow
            let monitor = Monitor::monitor_from_window_id(*self)?;
            Ok((monitor.dpi_x(), monitor.dpi_y()))
        }
    }

    // --- 位置 Getters ---

    #[inline]
    fn get_left(&self) -> Result<i32, Self::Error> {
        // 直接复用 get_window_rect 的逻辑
        self.get_window_rect().map(|v| v.0)
    }

    #[inline]
    fn get_top(&self) -> Result<i32, Self::Error> {
        self.get_window_rect().map(|v| v.1)
    }

    // --- 位置 Setters ---

    fn set_left(&self, left: i32) -> Result<(), Self::Error> {
        let (_, top, _, _) = self.get_window_rect()?;
        unsafe {
            SetWindowPos(
                self.hwnd(),
                None,
                left,
                top,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            Ok(())
        }
    }

    fn set_top(&self, top: i32) -> Result<(), Self::Error> {
        let (left, _, _, _) = self.get_window_rect()?;
        unsafe {
            SetWindowPos(
                self.hwnd(),
                None,
                left,
                top,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            Ok(())
        }
    }

    fn set_position(&self, pos: (i32, i32)) -> Result<(), Self::Error> {
        unsafe {
            SetWindowPos(
                self.hwnd(),
                None,
                pos.0,
                pos.1,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            Ok(())
        }
    }

    // --- 尺寸 Getters ---

    #[inline]
    fn get_width(&self) -> Result<u32, Self::Error> {
        self.get_window_rect().map(|v| v.2)
    }

    #[inline]
    fn get_height(&self) -> Result<u32, Self::Error> {
        self.get_window_rect().map(|v| v.3)
    }

    // --- 尺寸 Setters ---

    fn set_width(&self, width: u32) -> Result<(), Self::Error> {
        let (_, _, _, height) = self.get_window_rect()?;
        unsafe {
            SetWindowPos(
                self.hwnd(),
                None,
                0,
                0,
                width as i32,
                height as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            Ok(())
        }
    }

    fn set_height(&self, height: u32) -> Result<(), Self::Error> {
        let (_, _, width, _) = self.get_window_rect()?;
        unsafe {
            SetWindowPos(
                self.hwnd(),
                None,
                0,
                0,
                width as i32,
                height as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            Ok(())
        }
    }

    fn set_size(&self, size: (u32, u32)) -> Result<(), Self::Error> {
        unsafe {
            SetWindowPos(
                self.hwnd(),
                None,
                0,
                0,
                size.0 as i32,
                size.1 as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            Ok(())
        }
    }

    // --- 区域查询 ---

    fn get_client_size(&self) -> Result<(u32, u32), Self::Error> {
        unsafe {
            let mut rect = RECT::default();
            GetClientRect(self.hwnd(), &mut rect)?;
            // GetClientRect 的 left/top 永远是 0
            Ok((rect.right as u32, rect.bottom as u32))
        }
    }

    fn get_client_rect(&self) -> Result<(i32, i32, u32, u32), Self::Error> {
        unsafe {
            // 1. 获取大小
            let mut rect = RECT::default();
            GetClientRect(self.hwnd(), &mut rect)?;
            let width = rect.right as u32;
            let height = rect.bottom as u32;

            // 2. 获取屏幕位置
            // ClientToScreen 会把 rect 的 left/top (0,0) 转换成屏幕坐标
            let mut point = POINT { x: 0, y: 0 };
            ClientToScreen(self.hwnd(), &mut point).ok()?;

            Ok((point.x, point.y, width, height))
        }
    }

    fn get_window_rect(&self) -> Result<(i32, i32, u32, u32), Self::Error> {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.hwnd(), &mut rect)?;

            // rect.right/bottom 是坐标，不是宽高，需要相减
            let width = (rect.right - rect.left) as u32;
            let height = (rect.bottom - rect.top) as u32;

            Ok((rect.left, rect.top, width, height))
        }
    }

    fn drag_window(&self) -> Result<(), Self::Error> {
        unsafe {
            ReleaseCapture()?;
            SendMessageW(
                self.hwnd(),
                WM_SYSCOMMAND,
                Some(WPARAM((SC_MOVE | HTCAPTION) as usize)),
                Some(LPARAM(0)),
            );
            Ok(())
        }
    }

    fn set_ime_window_location(&self, rect: (i32, i32, u32, u32)) -> Result<(), Self::Error> {
        let (x, y, w, h) = rect;
        let hwnd = self.hwnd(); // 假设你能拿到 HWND

        unsafe {
            let h_imc = ImmGetContext(hwnd);
            if h_imc.is_invalid() {
                return Ok(()); // 或者返回 Error，视你策略而定
            }

            let mut form = COMPOSITIONFORM {
                dwStyle: CFS_POINT, // 使用点定位，但利用 rcArea 做避让参考
                ptCurrentPos: POINT { x, y },
                rcArea: RECT {
                    left: x,
                    top: y,
                    right: x + w as i32,
                    bottom: y + h as i32, // 【关键】告诉 IME 这里是底部，别遮挡
                },
            };

            // 发送给系统
            let _ = ImmSetCompositionWindow(h_imc, &mut form);

            // 别忘了释放
            let _ = ImmReleaseContext(hwnd, h_imc);
        }
        Ok(())
    }

    fn set_ime_open_state(&self, is_open: bool) -> Result<(), Self::Error> {
        let hwnd = self.hwnd();
        unsafe {
            let h_imc = ImmGetContext(hwnd);
            if !h_imc.is_invalid() {
                // true = 打开输入法, false = 关闭
                let _ = ImmSetOpenStatus(h_imc, is_open.into());
                let _ = ImmReleaseContext(hwnd, h_imc);
            }
        }
        Ok(())
    }

    fn set_ime_allowed(&self, allow: bool) -> Result<(), Self::Error> {
        unsafe {
            if allow {
                let h_imc = ImmGetContext(self.hwnd());
                dbg!(h_imc.is_invalid());
                dbg!(h_imc.0);
                ImmAssociateContext(self.hwnd(), h_imc);
            } else {
                ImmAssociateContext(self.hwnd(), HIMC::default());
            }
        }
        Ok(())
    }

    fn set_cursor(cursor: Option<Self::Cursor>) -> Result<(), Self::Error> {
        unsafe {
            SetCursor(cursor.map(|v| v.handle()));
        }
        Ok(())
    }

    fn destroy(&self) -> Result<(), Self::Error> {
        unsafe {
            DestroyWindow(self.hwnd())?;
            Ok(())
        }
    }
}

impl WindowOperations for WindowId {
    type Error = Error;

    fn request_redraw(&self) -> Result<(), Self::Error> {
        unsafe {
            InvalidateRect(Some(self.hwnd()), None, true).ok()?;
            Ok(())
        }
    }

    fn capture_mouse(&self) -> Result<(), Self::Error> {
        unsafe {
            SetCapture(self.hwnd());
        }
        Ok(())
    }

    fn release_mouse(&self) -> Result<(), Self::Error> {
        unsafe {
            ReleaseCapture()?;
        }
        Ok(())
    }
}
