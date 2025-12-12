use graphics::D2DBackendError;

#[derive(thiserror::Error, Debug)]
pub enum FlorRenderError {
    #[error("D2D renderer backend error: `{0}`")]
    D2DRendererBackendError(#[from] D2DBackendError),
    #[error("not found render instance")]
    RenderNotFound,
    #[error("Resource origin mismatch: Expected resources from current backend, found from different backend")]
    ResourceBackendMismatch,
}
