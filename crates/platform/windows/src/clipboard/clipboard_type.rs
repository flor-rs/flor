use flor_base::platform::ClipboardType;
use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
use windows::Win32::System::Ole::{CF_DIB, CF_HDROP, CF_UNICODETEXT};
use windows_core::w;

#[derive(Debug, Copy, Clone)]
pub struct RawType(pub u32);

impl From<ClipboardType> for RawType {
    fn from(t: ClipboardType) -> Self {
        Self(match t {
            ClipboardType::Text => CF_UNICODETEXT.0 as u32,
            ClipboardType::Html => unsafe { RegisterClipboardFormatW(w!("HTML Format")) },
            ClipboardType::Image => CF_DIB.0 as u32,
            ClipboardType::Rtf => unsafe { RegisterClipboardFormatW(w!("Rich Text Format")) },
            ClipboardType::Files => CF_HDROP.0 as u32,
        } as u32)
    }
}
