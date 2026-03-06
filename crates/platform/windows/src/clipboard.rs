use flor_base::platform::{ClipboardApi, ClipboardType};
use std::ptr::copy_nonoverlapping;
use std::slice;
use windows::Win32::Foundation::*;
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows_core::PCWSTR;

mod clipboard_type;
mod error;

use crate::conversions::encode_wide::encode_wide;
use crate::WindowId;
pub use clipboard_type::*;
pub use error::*;

pub struct Clipboard(pub Option<WindowId>);

struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

impl ClipboardApi for Clipboard {
    // Windows 下原生格式标识是 u32
    type RawClipboardType = RawType;
    type Error = Error;

    fn register_format(&self, name: &str) -> Result<Self::RawClipboardType, Self::Error> {
        let utf16 = encode_wide(name);

        let id = unsafe { RegisterClipboardFormatW(PCWSTR(utf16.as_ptr())) };

        if id == 0 {
            return Err(Error::System(windows_core::Error::from_thread()));
        }

        Ok(RawType(id))
    }

    fn set_clipboard_text(&self, content: String) -> Result<(), Self::Error> {
        let utf16 = encode_wide(content);
        let slice = unsafe { slice::from_raw_parts(utf16.as_ptr() as *const u8, utf16.len() * 2) };

        self.set_clipboard(slice, ClipboardType::Text)
    }

    fn get_clipboard_text(&self) -> Result<String, Self::Error> {
        unsafe {
            OpenClipboard(self.0.map(|w| w.hwnd()))?;
            let _guard = ClipboardGuard;

            let h_mem = HGLOBAL(GetClipboardData(CF_UNICODETEXT.0 as u32)?.0);

            let ptr = GlobalLock(h_mem) as *const u16;

            if ptr.is_null() {
                return Err(Self::Error::ClipboardIsEmpty);
            }

            // 计算字符串长度（找 \0）
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }

            let utf16_slice = slice::from_raw_parts(ptr, len);
            let result = String::from_utf16_lossy(utf16_slice);

            let _ = GlobalUnlock(h_mem);

            Ok(result)
        }
    }

    fn set_clipboard(
        &self,
        data: &[u8],
        r#type: impl Into<Self::RawClipboardType>,
    ) -> Result<(), Self::Error> {
        unsafe {
            OpenClipboard(self.0.map(|w| w.hwnd()))?;
            let _guard = ClipboardGuard;

            EmptyClipboard()?;
            Self::write_to_handle(r#type.into().0, data)?;

            Ok(())
        }
    }

    fn set_clipboard_muti_type(
        &self,
        data: &[(&[u8], impl Into<Self::RawClipboardType> + Copy)],
    ) -> Result<(), Self::Error> {
        unsafe {
            OpenClipboard(self.0.map(|w| w.hwnd()))?;
            let _guard = ClipboardGuard;

            EmptyClipboard()?;
            for (bytes, r#type) in data {
                Self::write_to_handle(r#type.clone().into().0, bytes)?;
            }

            Ok(())
        }
    }

    fn get_clipboard(
        &self,
        r#type: impl Into<Self::RawClipboardType>,
    ) -> Result<Vec<u8>, Self::Error> {
        let fmt = r#type.into();
        unsafe {
            OpenClipboard(self.0.map(|w| w.hwnd()))?;
            let _guard = ClipboardGuard;

            let h_mem = HGLOBAL(GetClipboardData(fmt.0)?.0 as _);
            let size = GlobalSize(h_mem);
            let ptr = GlobalLock(h_mem) as *const u8;

            if ptr.is_null() {
                return Err(Self::Error::ClipboardIsEmpty);
            }

            let mut result = vec![0u8; size];
            copy_nonoverlapping(ptr, result.as_mut_ptr(), size);

            let _ = GlobalUnlock(h_mem);

            Ok(result)
        }
    }
}

impl Clipboard {
    unsafe fn write_to_handle(u_format: u32, data: &[u8]) -> Result<(), Error> {
        if data.is_empty() {
            return Ok(());
        }

        let h_mem = GlobalAlloc(GMEM_MOVEABLE, data.len())?;
        let dest_ptr = GlobalLock(h_mem);

        if dest_ptr.is_null() {
            GlobalFree(Some(h_mem))?;
            return Err(Error::System(windows_core::Error::from_thread()));
        }

        copy_nonoverlapping(data.as_ptr(), dest_ptr as *mut u8, data.len());
        let _ = GlobalUnlock(h_mem);

        if let Err(e) = SetClipboardData(u_format, Some(HANDLE(h_mem.0))) {
            GlobalFree(Some(h_mem))?;
            return Err(Error::System(e));
        }

        Ok(())
    }
}
