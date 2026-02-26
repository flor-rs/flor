use libblur::BlurError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TinySkiaError {
    #[cfg(target_os = "windows")]
    #[error("window core error: `{0}`")]
    WindowCore(#[from] windows::core::Error),
    #[error("failed to create surface")]
    CreateSurfaceError,
    #[error("surface not found")]
    SurfaceNotFoundError,
    #[error("failed to decode image")]
    ImageDecodeError,
    #[error("invalid image format")]
    ImageInvalidFormat,
    #[error("blur error")]
    BlurError(#[from] BlurError),
}
