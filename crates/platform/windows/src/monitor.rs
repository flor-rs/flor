use crate::Error;
use flor_platform_base::MonitorApi;
use windows::Win32::Foundation::{LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint, HDC, HMONITOR, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows_core::BOOL;

#[derive(Debug, Clone)]
pub struct Monitor {
    pub h_monitor: HMONITOR, // Win32 HMONITOR
    pub name: String,        // 设备名 (如 \\.\DISPLAY1)
    pub is_primary: bool,
    pub rect: (f32, f32, u32, u32),      // 屏幕物理像素坐标
    pub work_area: (f32, f32, u32, u32), // 除去任务栏的区域
    pub scale_factor: f32,               // DPI 缩放
}

unsafe impl Send for Monitor {}
unsafe impl Sync for Monitor {}

impl MonitorApi for Monitor {
    type Monitor = Self;
    type Error = Error;

    fn enumerate_monitors() -> Result<Vec<Self::Monitor>, Self::Error> {
        let mut monitors: Vec<Monitor> = Vec::new();

        unsafe {
            // 核心魔法：把 monitors 的裸指针传给回调函数
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
            // 1. 直接拿句柄
            let h_monitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);

            // 2. 复用逻辑：直接调用辅助函数
            // 如果获取失败（极少见），返回系统错误
            Monitor::from_handle(h_monitor)
                .ok_or_else(|| Error::from(Error::from_win32()))
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

    fn inner(self) -> Self::Monitor {
        self
    }
}

impl Monitor {
    // 【新增】私有辅助函数：封装通用的解析逻辑
    // 无论是遍历还是单点查询，最后都调用它
    unsafe fn from_handle(h_monitor: HMONITOR) -> Option<Self> {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;

        // 获取信息失败直接返回 None
        if !GetMonitorInfoW(h_monitor, &mut info.monitorInfo as *mut _ as *mut _).as_bool() {
            return None;
        }

        let rc = info.monitorInfo.rcMonitor;
        let work = info.monitorInfo.rcWork;
        let name = String::from_utf16_lossy(
            &info.szDevice.split(|&c| c == 0).next().unwrap_or(&[])
        );
        let is_primary = (info.monitorInfo.dwFlags & 1) != 0;

        let mut dpi_x = 0;
        let mut dpi_y = 0;
        let _ = GetDpiForMonitor(h_monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
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
        })
    }
}

unsafe extern "system" fn monitor_enum_proc(
    h_monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    // 1. 把 lparam 还原回 Vec 指针
    let monitors = &mut *(lparam.0 as *mut Vec<Monitor>);

    // 复用逻辑：调用辅助函数，成功则 push
    if let Some(m) = Monitor::from_handle(h_monitor) {
        monitors.push(m);
    }

    // 返回 TRUE 继续枚举下一个屏幕
    BOOL(1)
}
