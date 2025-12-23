use std::ffi::c_void;
use windows::core::*;
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Graphics::DirectWrite::*;

#[implement(IDWriteFontFileStream)]
pub struct MemoryFontFileStream {
    pub data: Vec<u8>,
}

impl IDWriteFontFileStream_Impl for MemoryFontFileStream_Impl {
    fn ReadFileFragment(
        &self,
        fragment_start: *mut *mut c_void,
        file_offset: u64,
        fragment_size: u64,
        _fragment_context: *mut *mut c_void,
    ) -> Result<()> {
        let start = file_offset as usize;
        let end = start + fragment_size as usize;

        if end > self.data.len() {
            return Err(Error::from_hresult(E_INVALIDARG));
        }

        unsafe {
            *fragment_start = self.data[start..end].as_ptr() as *mut c_void;
        }
        Ok(())
    }

    fn ReleaseFileFragment(&self, _fragment_context: *mut c_void) {
        // Rust Vec 管理内存，无需手动释放片段
    }

    fn GetFileSize(&self) -> Result<u64> {
        Ok(self.data.len() as u64)
    }

    fn GetLastWriteTime(&self) -> Result<u64> {
        Ok(0)
    }
}