use graphics::base::BrushHandle;

#[cfg(feature = "direct2d")]
use graphics::handle::D2DBrushHandle;
#[cfg(feature = "opengl")]
use graphics::handle::GlBrushHandle;

#[derive(Debug, Clone)]
pub enum FlorBrushHandle {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DBrushHandle,
        #[cfg(feature = "opengl")] GlBrushHandle,
    ),
}

impl BrushHandle for FlorBrushHandle {}
