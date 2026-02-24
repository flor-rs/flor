#[derive(thiserror::Error, Debug)]
pub enum FlorRenderError {
    #[cfg(feature = "direct2d")]
    #[error("D2D renderer backend error: `{0}`")]
    D2DRendererBackendError(#[from] graphics::error::D2DBackendError),
    #[cfg(feature = "opengl")]
    #[error("GL renderer backend error: `{0}`")]
    GlRendererBackendError(#[from] graphics::error::GlRendererError),
    #[error("not found render instance")]
    RenderNotFound,
    #[error("Resource origin mismatch: Expected resources from current backend, found from different backend")]
    ResourceBackendMismatch,
}
