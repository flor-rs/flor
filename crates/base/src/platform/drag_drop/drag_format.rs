#[derive(Debug, Clone, PartialEq)]
pub enum DragFormat {
    /// 对应 CF_HDROP (Win), public.file-url (Mac), text/uri-list (Linux)
    Files(String),
    /// 对应 CF_TEXT (Win), public.utf8-plain-text (Mac), text/plain (Linux)
    Text(String),
    /// 对应 CF_DIB (Win), public.image (Mac), image/png (Linux)
    Image(String),
    /// 某些特定平台的原生格式，用于扩展
    Custom(String),
}
