use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

#[inline]
pub fn encode_unicode(str: impl AsRef<OsStr>) -> Vec<u16> {
    str.as_ref().encode_wide().chain(once(0)).collect()
}

// pub fn decode_wide(mut wide_c_string: &[u16]) -> OsString {
//     if let Some(null_pos) = wide_c_string.iter().position(|c| *c == 0) {
//         wide_c_string = &wide_c_string[..null_pos];
//     }
//
//     OsString::from_wide(wide_c_string)
// }

#[inline]
pub fn encode_ansi(str: impl AsRef<OsStr>) -> Vec<u8> {
    let mut pc_str = str.as_ref().to_string_lossy().to_string().into_bytes();
    pc_str.push(0); // 添加 null 字节
    pc_str
}