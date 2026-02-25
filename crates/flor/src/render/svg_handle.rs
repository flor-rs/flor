#[cfg(feature = "tiny-skia")]
use crate::graphics_cpu::handle::TinySkiaSvgHandle;
#[cfg(feature = "direct2d")]
use crate::graphics_gpu::handle::D2DSvgHandle;
#[cfg(feature = "opengl")]
use crate::graphics_gpu::handle::GlSvgHandle;
use flor_base::graphics::SvgHandle;

#[derive(Debug, Clone)]
pub enum FlorSvgHandle {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DSvgHandle,
        #[cfg(feature = "opengl")] GlSvgHandle,
    ),
    #[cfg(feature = "cpu-render-backend")]
    CPU(#[cfg(feature = "tiny-skia")] TinySkiaSvgHandle),
}

impl SvgHandle for FlorSvgHandle {}
