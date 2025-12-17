use crate::render::backend_error::FlorRenderError;
use crate::view::view_id::ViewId;
use flor_graphics_base::ColorParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("无效的view_id : `{0}`")]
    ControlUnregistered(ViewId),
    #[error("graphics error: `{0}`")]
    GraphicsError(#[from] FlorRenderError),
    #[error("color parse error: `{0}`")]
    ColorParseColor(#[from] ColorParseError),
    #[error("taffy error: `{0}`")]
    TaffyError(#[from] taffy::TaffyError),
    #[error("init error: `{0}`")]
    InitError(String),
    #[error("render backend error: `{0}`")]
    RenderBackendError(#[from] Box<dyn std::error::Error>),
    #[cfg(windows)]
    #[error("window operations error: `{0}`")]
    WindowError(#[from] platform::Error),
    #[cfg(feature = "clipboard")]
    #[error("clipboard error: `{0}`")]
    ClipboardError(#[from] arboard::Error),
}
