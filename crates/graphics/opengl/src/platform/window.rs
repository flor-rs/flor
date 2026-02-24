use crate::error::GlRendererError;
use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;
use std::ptr::null;
use std::sync::OnceLock;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Gdi::GetDC;
use windows::Win32::Graphics::OpenGL::{
    wglCreateContext, wglGetProcAddress, wglMakeCurrent, ChoosePixelFormat, SetPixelFormat,
    PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

static OPENGL32_LIB: OnceLock<usize> = OnceLock::new();

pub fn get_gl_context(h_wnd: HWND) -> Result<glow::Context, GlRendererError> {
    unsafe {
        // 1. 获取设备上下文
        let hdc = GetDC(Some(h_wnd));

        // 2. 配置像素格式
        let mut pfd: PIXELFORMATDESCRIPTOR = std::mem::zeroed();
        pfd.nSize = size_of::<PIXELFORMATDESCRIPTOR>() as u16;
        pfd.nVersion = 1;
        pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
        pfd.iPixelType = PFD_TYPE_RGBA;
        pfd.cColorBits = 32;
        pfd.cDepthBits = 24;
        pfd.cStencilBits = 8;
        pfd.iLayerType = 0; // PFD_MAIN_PLANE

        let format = ChoosePixelFormat(hdc, &pfd);
        SetPixelFormat(hdc, format, &pfd)?;

        // 3. 创建并激活 HGLRC
        let h_gl_rc = wglCreateContext(hdc)?;
        wglMakeCurrent(hdc, h_gl_rc)?;

        // 4. 手动绑定 glow
        let gl = glow::Context::from_loader_function_cstr(|addr| {
            let fn_ptr = wglGetProcAddress(PCSTR(addr.as_ptr() as *const _));

            // 非空返回
            if let Some(fn_ptr) = fn_ptr {
                return fn_ptr as *const _;
            }

            let lib_opengl32 = if let Some(x) = OPENGL32_LIB.get() {
                HMODULE(*x as _)
            } else {
                let name = OsStr::new("opengl32.dll")
                    .encode_wide()
                    .chain(Some(0))
                    .collect::<Vec<_>>();
                let lib_addr =
                    if let Ok(lib_opengl32) = LoadLibraryW(PCWSTR(name.as_ptr() as *const _)) {
                        lib_opengl32
                    } else {
                        return null() as *const _;
                    };
                let _ = OPENGL32_LIB.set(lib_addr.0 as usize);
                lib_addr
            };

            GetProcAddress(lib_opengl32, PCSTR(addr.as_ptr() as *const _))
                .map_or(null(), |fn_ptr| fn_ptr as *const _)
        });
        Ok(gl)
    }
}

// 1. 定义函数指针类型（这是扩展标准定义的签名）
type WglSwapIntervalEXT = unsafe extern "system" fn(i32) -> i32;

pub fn set_vsync(enabled: bool) {
    unsafe {
        // 2. 这里的字符串必须精确匹配扩展名
        let name = std::ffi::CString::new("wglSwapIntervalEXT").unwrap();

        // 3. 询问驱动程序：你支持这个功能吗？如果支持，给我函数地址
        let addr = windows::Win32::Graphics::OpenGL::wglGetProcAddress(windows::core::PCSTR(
            name.as_ptr() as *const _,
        ));

        if let Some(addr) = addr {
            let func: WglSwapIntervalEXT = std::mem::transmute(addr);
            // 参数 1 表示同步到 1 个垂直刷新周期（开启），0 表示立即交换（关闭）
            func(if enabled { 1 } else { 0 });
        } else {
            // 驱动不支持 WGL_EXT_swap_control 扩展
            eprintln!("VSync extension not supported by the graphics driver.");
        }
    }
}
