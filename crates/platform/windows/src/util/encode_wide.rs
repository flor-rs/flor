use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

pub fn encode_wide(string: impl AsRef<OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(once(0)).collect()
}

// pub fn decode_wide(mut wide_c_string: &[u16]) -> OsString {
//     if let Some(null_pos) = wide_c_string.iter().position(|c| *c == 0) {
//         wide_c_string = &wide_c_string[..null_pos];
//     }
//
//     OsString::from_wide(wide_c_string)
// }