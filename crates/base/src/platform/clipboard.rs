pub trait ClipboardApi {
    type RawClipboardType: Copy;
    type Error;
    fn register_format(&self, name: &str) -> Result<Self::RawClipboardType, Self::Error>;
    fn set_clipboard_text(&self, content: String) -> Result<(), Self::Error>;
    fn get_clipboard_text(&self) -> Result<String, Self::Error>;
    fn set_clipboard(
        &self,
        data: &[u8],
        r#type: impl Into<Self::RawClipboardType>,
    ) -> Result<(), Self::Error>;
    fn set_clipboard_muti_type(
        &self,
        data: &[(&[u8], impl Into<Self::RawClipboardType> + Copy)],
    ) -> Result<(), Self::Error>;
    fn get_clipboard(
        &self,
        r#type: impl Into<Self::RawClipboardType>,
    ) -> Result<Vec<u8>, Self::Error>;
}

#[derive(Debug, Copy, Clone)]
pub enum ClipboardType {
    Text,
    Image,
    Rtf,
    Html,
    Files,
}
