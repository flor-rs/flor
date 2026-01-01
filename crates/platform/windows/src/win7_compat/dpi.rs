use windows::core::s;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSX};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
#[cfg(all(not(feature = "monitor"), feature = "win7-compat"))]
pub unsafe fn get_dpi_compatible(hwnd: HWND) -> u32 {
    // 1. 尝试动态加载 Win10 的 GetDpiForWindow
    //    场景：用户开启了兼容模式编译，但实际运行在 Win10/11 上
    if let Ok(h_module) = GetModuleHandleA(s!("user32.dll")) {
        if let Some(func_ptr) = GetProcAddress(h_module, s!("GetDpiForWindow")) {
            // 定义函数签名: UINT (HWND)
            type FuncType = unsafe extern "system" fn(windows::Win32::Foundation::HWND) -> u32;
            let func: FuncType = std::mem::transmute(func_ptr);

            let dpi = func(hwnd);
            if dpi != 0 {
                return dpi;
            }
        }
    }

    // 2. Win7 保底策略：读取系统全局 DPI (System DPI)
    //    Win7 不支持针对特定窗口的不同 DPI，所以读系统值是正确的
    let hdc = GetDC(Some(hwnd));
    let dpi = if !hdc.is_invalid() {
        let d = GetDeviceCaps(Some(hdc), LOGPIXELSX) as u32;
        ReleaseDC(Some(hwnd), hdc);
        d
    } else {
        96
    };

    if dpi > 0 {
        dpi
    } else {
        96
    }
}
