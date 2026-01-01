use crate::{Error, WindowId};
use flor_platform_base::MonitorApi;
use std::mem::size_of;
use windows::core::BOOL;
use windows::Win32::Foundation::{LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, HDC, HMONITOR,
    MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
};

#[derive(Debug, Clone)]
pub struct Monitor {
    pub h_monitor: HMONITOR,
    pub name: String,
    pub is_primary: bool,
    pub rect: (f32, f32, u32, u32),
    pub work_area: (f32, f32, u32, u32),
    pub scale_factor: f32,
    pub dpi_x: u32,
    pub dpi_y: u32,
}

unsafe impl Send for Monitor {}
unsafe impl Sync for Monitor {}

impl MonitorApi for Monitor {
    type Monitor = Self;
    type Error = Error;
    type WindowId = WindowId;

    fn enumerate_monitors() -> Result<Vec<Self::Monitor>, Self::Error> {
        let mut monitors: Vec<Monitor> = Vec::new();
        unsafe {
            let _ = EnumDisplayMonitors(
                None,
                None,
                Some(monitor_enum_proc),
                LPARAM(&mut monitors as *mut _ as isize),
            );
        }
        Ok(monitors)
    }

    fn monitor_from_point(x: i32, y: i32) -> Result<Self::Monitor, Self::Error> {
        unsafe {
            let pt = POINT { x, y };
            let h_monitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
            Monitor::from_handle(h_monitor)
                .ok_or_else(|| Error::from(windows::core::Error::from_win32()))
        }
    }

    fn monitor_from_window_id(window_id: WindowId) -> Result<Self::Monitor, Self::Error> {
        unsafe {
            let hwnd = window_id.hwnd();
            let h_monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            Monitor::from_handle(h_monitor)
                .ok_or_else(|| Error::from(windows::core::Error::from_win32()))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
    fn is_primary(&self) -> bool {
        self.is_primary
    }
    fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
    fn rect(&self) -> (f32, f32, u32, u32) {
        self.rect
    }
    fn work_area(&self) -> (f32, f32, u32, u32) {
        self.work_area
    }
    fn dpi_x(&self) -> f64 {
        self.dpi_x as f64
    }
    fn dpi_y(&self) -> f64 {
        self.dpi_y as f64
    }
    fn inner(self) -> Self::Monitor {
        self
    }
}

impl Monitor {
    unsafe fn from_handle(h_monitor: HMONITOR) -> Option<Self> {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;

        if !GetMonitorInfoW(h_monitor, &mut info.monitorInfo as *mut _ as *mut _).as_bool() {
            return None;
        }

        let rc = info.monitorInfo.rcMonitor;
        let work = info.monitorInfo.rcWork;
        let name =
            String::from_utf16_lossy(&info.szDevice.split(|&c| c == 0).next().unwrap_or(&[]));
        let is_primary = (info.monitorInfo.dwFlags & 1) != 0;

        // 核心修改：使用封装的兼容性函数获取 DPI
        let (dpi_x, dpi_y) = get_monitor_dpi(h_monitor);

        let scale = if dpi_x > 0 { dpi_x as f32 / 96.0 } else { 1.0 };

        Some(Self {
            h_monitor,
            name,
            is_primary,
            rect: (
                rc.left as f32,
                rc.top as f32,
                (rc.right - rc.left) as u32,
                (rc.bottom - rc.top) as u32,
            ),
            work_area: (
                work.left as f32,
                work.top as f32,
                (work.right - work.left) as u32,
                (work.bottom - work.top) as u32,
            ),
            scale_factor: scale,
            dpi_x,
            dpi_y,
        })
    }
}

unsafe extern "system" fn monitor_enum_proc(
    h_monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<Monitor>);
    if let Some(m) = Monitor::from_handle(h_monitor) {
        monitors.push(m);
    }
    BOOL(1)
}

unsafe fn get_monitor_dpi(h_monitor: HMONITOR) -> (u32, u32) {
    // [方案 1] 现代高性能路径 (Win8.1 / Win10 / Win11)
    // 直接静态链接，速度最快
    #[cfg(not(feature = "win7-compat"))]
    {
        use windows::Win32::UI::HiDpi::GetDpiForMonitor;
        use windows::Win32::UI::HiDpi::MDT_EFFECTIVE_DPI;

        let mut x = 96;
        let mut y = 96;
        // 如果失败保持默认 96
        let _ = GetDpiForMonitor(h_monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);
        (x, y)
    }

    // [方案 2] Win7 兼容路径 (Dynamic Loading + GDI Fallback)
    #[cfg(feature = "win7-compat")]
    {
        crate::win7_compat::get_monitor_dpi(h_monitor)
    }
}
