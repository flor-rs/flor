#[derive(thiserror::Error, Debug)]
pub enum FlorRendererError {
    #[cfg(feature = "direct2d")]
    #[error("D2D renderer backend error: `{0}`")]
    D2D(#[from] graphics::error::D2DError),
    #[cfg(feature = "opengl")]
    #[error("GL renderer backend error: `{0}`")]
    Gl(#[from] graphics::error::GlError),
    #[error("not found render instance")]
    RenderNotFound,
    #[error("Resource origin mismatch: Expected resources from current backend, found from different backend")]
    ResourceMismatch,
}
