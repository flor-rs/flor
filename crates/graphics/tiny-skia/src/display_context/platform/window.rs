use crate::display_context::DisplayContext;
use crate::error::TinySkiaError;
use crate::TinySkiaConfig;
use std::fmt::{Debug, Formatter};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DwmFlush;
use windows::Win32::Graphics::Gdi::{
    GetDC, ReleaseDC, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, HDC, SRCCOPY,
};

pub struct GdiDisplayContext {
    pub h_wnd: HWND,
    pub hdc: HDC,
}

unsafe impl Send for GdiDisplayContext {}
unsafe impl Sync for GdiDisplayContext {}

impl Debug for GdiDisplayContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TinySkiaContext")
            .field("h_wnd", &self.h_wnd.0)
            .field("hdc", &self.hdc.0)
            .finish()
    }
}

impl DisplayContext for GdiDisplayContext {
    type HWND = HWND;
    fn create(h_wnd: HWND, _config: TinySkiaConfig) -> Result<Self, TinySkiaError>
    where
        Self: Sized,
    {
        unsafe {
            // According to windows-rs, GetDC may take Option<HWND> or HWND directly depending on version.
            // We'll try with window handle wrapped in Some.
            let hdc = GetDC(Some(h_wnd));
            Ok(GdiDisplayContext { h_wnd, hdc })
        }
    }
    fn present(&mut self, width: u32, height: u32, pixels: &[u8]) -> Result<(), TinySkiaError> {
        #[repr(C)]
        #[allow(non_camel_case_types)]
        struct BITMAPINFO_RGBA {
            bmi_header: BITMAPINFOHEADER,
            bmi_colors: [u32; 3],
        }

        let mut bmi = BITMAPINFO_RGBA {
            bmi_header: BITMAPINFOHEADER::default(),
            bmi_colors: [0x000000FF, 0x0000FF00, 0x00FF0000], // R, G, B masks
        };

        bmi.bmi_header.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmi_header.biWidth = width as i32;
        bmi.bmi_header.biHeight = -(height as i32); // Top-down
        bmi.bmi_header.biPlanes = 1;
        bmi.bmi_header.biBitCount = 32;
        bmi.bmi_header.biCompression = windows::Win32::Graphics::Gdi::BI_BITFIELDS.0;

        unsafe {
            StretchDIBits(
                self.hdc,
                0,
                0,
                width as i32,
                height as i32,
                0,
                0,
                width as i32,
                height as i32,
                Some(pixels.as_ptr() as *const _),
                &bmi as *const BITMAPINFO_RGBA as *const BITMAPINFO,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
        Ok(())
    }

    fn wait_v_sync(&self) {
        unsafe {
            let _ = DwmFlush();
        }
    }
}

impl Drop for GdiDisplayContext {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseDC(Some(self.h_wnd), self.hdc);
        }
    }
}
