mod memory_font_file_loader;
mod memory_font_file_stream;

pub use memory_font_file_loader::*;
use std::ffi::c_void;
use std::slice;
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::Graphics::DirectWrite::IDWriteFontFace;
use windows_core::{Error, BOOL};

/// 从 IDWriteFontFace 中手动解析 Family Name
/// OpenType 规范: https://learn.microsoft.com/en-us/typography/opentype/spec/name
pub unsafe fn get_family_name_from_face(face: &IDWriteFontFace) -> Result<String, Error> {
    let mut table_context: *mut c_void = std::ptr::null_mut();
    let mut table_data_ptr: *mut c_void = std::ptr::null_mut();
    let mut table_size: u32 = 0;
    let mut exists = BOOL(0);

    // 'name' 表的 Tag (BE: 0x6E616D65 -> LE: 0x656D616E)
    // 实际上 DWrite 期望的是 LE 整数，所以是 'n' | 'a'<<8 | 'm'<<16 | 'e'<<24
    let name_tag = u32::from_le_bytes(*b"name");

    face.TryGetFontTable(
        name_tag,
        &mut table_data_ptr,
        &mut table_size,
        &mut table_context,
        &mut exists,
    )?;

    if exists.0 == 0 || table_data_ptr.is_null() {
        return Err(Error::from_hresult(E_FAIL));
    }

    // 确保退出时释放 Table Context
    struct TableGuard<'a> {
        face: &'a IDWriteFontFace,
        context: *mut c_void,
    }
    impl<'a> Drop for TableGuard<'a> {
        fn drop(&mut self) {
            unsafe { self.face.ReleaseFontTable(self.context) };
        }
    }
    let _guard = TableGuard {
        face,
        context: table_context,
    };

    // 开始解析二进制数据
    let data = slice::from_raw_parts(table_data_ptr as *const u8, table_size as usize);

    // 简单的 Helper 读取 Big Endian u16
    let read_u16 = |offset: usize| -> Option<u16> {
        if offset + 2 > data.len() {
            return None;
        }
        Some(u16::from_be_bytes([data[offset], data[offset + 1]]))
    };

    let count = read_u16(2).ok_or(Error::from_hresult(E_FAIL))?;
    let string_offset = read_u16(4).ok_or(Error::from_hresult(E_FAIL))? as usize;

    // 遍历 Name Records
    // Record 结构: platformID(2), encodingID(2), languageID(2), nameID(2), length(2), offset(2)
    // Record 大小 = 12 字节，从 offset 6 开始
    for i in 0..count {
        let base = 6 + (i as usize * 12);

        let platform_id = read_u16(base).unwrap_or(0);
        let name_id = read_u16(base + 6).unwrap_or(0);

        // 我们只找 Windows Platform (3) 的 Font Family Name (1)
        // 注意：如果你需要更健壮的匹配（如 Preferred Family Name ID 16），可以在这里添加逻辑
        if platform_id == 3 && name_id == 1 {
            let length = read_u16(base + 8).unwrap_or(0) as usize;
            let offset = read_u16(base + 10).unwrap_or(0) as usize;

            let str_start = string_offset + offset;
            let str_end = str_start + length;

            if str_end <= data.len() {
                // Windows 平台的字符串通常是 UTF-16BE
                let name_bytes = &data[str_start..str_end];
                let u16_vec: Vec<u16> = name_bytes
                    .chunks_exact(2)
                    .map(|c| u16::from_be_bytes([c[0], c[1]]))
                    .collect();

                if let Ok(name) = String::from_utf16(&u16_vec) {
                    // 找到了直接返回
                    return Ok(name);
                }
            }
        }
    }

    Err(Error::from_hresult(E_FAIL)) // 没找到名字
}
