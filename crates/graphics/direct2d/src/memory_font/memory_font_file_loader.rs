use crate::memory_font::memory_font_file_stream::MemoryFontFileStream;
use std::ffi::c_void;
use windows::core::Result;
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFontFileLoader, IDWriteFontFileLoader_Impl, IDWriteFontFileStream,
};
use windows_core::implement;

#[derive(Default)]
#[implement(IDWriteFontFileLoader)]
pub struct MemoryFontFileLoader {
    pub data: Vec<u8>,
}

impl IDWriteFontFileLoader_Impl for MemoryFontFileLoader_Impl {
    #[allow(non_snake_case)]
    fn CreateStreamFromKey(
        &self,
        _font_file_reference_key: *const c_void,
        _font_file_reference_key_size: u32,
    ) -> Result<IDWriteFontFileStream> {
        // 创建一个新的 Stream 实例，克隆数据（或者使用 Arc 共享以减少内存拷贝）
        // 这里为了代码简洁直接 clone，性能敏感可改用 Arc<Vec<u8>>
        let stream = MemoryFontFileStream {
            data: self.data.clone(),
        };
        Ok(stream.into())
    }
}
