use flor_base::graphics::BrushHandle;

#[cfg(feature = "tiny-skia")]
use crate::graphics_cpu::handle::TinySkiaBrushHandle;
#[cfg(feature = "direct2d")]
use crate::graphics_gpu::handle::D2DBrushHandle;
#[cfg(feature = "opengl")]
use crate::graphics_gpu::handle::GlBrushHandle;

#[derive(Debug, Clone)]
pub enum FlorBrushHandle {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DBrushHandle,
        #[cfg(feature = "opengl")] GlBrushHandle,
    ),
    #[cfg(feature = "cpu-render-backend")]
    CPU(#[cfg(feature = "tiny-skia")] TinySkiaBrushHandle),
}

impl BrushHandle for FlorBrushHandle {}
