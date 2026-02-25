#[derive(thiserror::Error, Debug)]
pub enum FlorRendererError {
    #[cfg(feature = "direct2d")]
    #[error("D2D renderer backend error: `{0}`")]
    D2D(#[from] crate::graphics_gpu::error::D2DError),
    #[cfg(feature = "opengl")]
    #[error("GL renderer backend error: `{0}`")]
    Gl(#[from] crate::graphics_gpu::error::GlError),
    #[cfg(feature = "tiny-skia")]
    #[error("TinySkia renderer backend error: `{0}`")]
    TinySkia(#[from] crate::graphics_cpu::error::TinySkiaError),
    #[error("not found render instance")]
    RenderNotFound,
    #[error("Resource origin mismatch: Expected resources from current backend, found from different backend")]
    ResourceMismatch,
}
