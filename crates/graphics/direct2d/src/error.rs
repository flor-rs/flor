use thiserror::Error;

#[derive(Error, Debug)]
pub enum D2DBackendError {
    #[error("window core error: `{0}`")]
    WindowCore(#[from] windows::core::Error),
    #[error("renderer error: `{0}`")]
    RendererBaseError(#[from] flor_graphics_base::Error),
}
