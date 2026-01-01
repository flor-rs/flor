use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, HMONITOR, LOGPIXELSX};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::UI::HiDpi::{MDT_EFFECTIVE_DPI, MONITOR_DPI_TYPE};
use windows_core::s;

// 辅助：动态加载 GetDpiForMonitor
pub unsafe fn try_dynamic_get_dpi_for_monitor(h_monitor: HMONITOR) -> Option<(u32, u32)> {
    if let Ok(h_module) = LoadLibraryA(s!("shcore.dll")) {
        let func_ptr = GetProcAddress(h_module, s!("GetDpiForMonitor"));
        if let Some(func) = func_ptr {
            // 定义函数签名
            type GetDpiFunc = unsafe extern "system" fn(HMONITOR, MONITOR_DPI_TYPE, *mut u32, *mut u32) -> windows::core::HRESULT;
            let func: GetDpiFunc = std::mem::transmute(func);

            let mut x = 0;
            let mut y = 0;
            if func(h_monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y).is_ok() {
                return Some((x, y));
            }
        }
    }
    None
}

pub unsafe fn get_monitor_dpi(h_monitor: HMONITOR) -> (u32, u32) {
    use crate::win7_compat::try_dynamic_get_dpi_for_monitor;
    if let Some((x, y)) = try_dynamic_get_dpi_for_monitor(h_monitor) {
        return (x, y);
    }

    let hdc = GetDC(None); // 获取整个屏幕的 DC
    if !hdc.is_invalid() {
        let dpi = GetDeviceCaps(Some(hdc), LOGPIXELSX) as u32;
        ReleaseDC(None, hdc);
        return (dpi, dpi);
    }
    (96, 96)
}