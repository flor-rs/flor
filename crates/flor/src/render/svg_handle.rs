use flor_base::graphics::SvgHandle;
#[cfg(feature = "direct2d")]
use graphics::handle::D2DSvgHandle;
#[cfg(feature = "opengl")]
use graphics::handle::GlSvgHandle;

#[derive(Debug, Clone)]
pub enum FlorSvgHandle {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DSvgHandle,
        #[cfg(feature = "opengl")] GlSvgHandle,
    ),
}

impl SvgHandle for FlorSvgHandle {}
