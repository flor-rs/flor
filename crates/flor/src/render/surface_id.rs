#[cfg(feature = "tiny-skia")]
use crate::graphics_cpu::handle::TinySkiaSurfaceId;
#[cfg(feature = "direct2d")]
use crate::graphics_gpu::handle::D2DSurfaceId;
#[cfg(feature = "opengl")]
use crate::graphics_gpu::handle::GlSurfaceId;
use flor_base::graphics::SurfaceId;

#[derive(Debug, Clone)]
pub enum FlorSurfaceId {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DSurfaceId,
        #[cfg(feature = "opengl")] GlSurfaceId,
    ),
    #[cfg(feature = "cpu-render-backend")]
    CPU(#[cfg(feature = "tiny-skia")] TinySkiaSurfaceId),
}
impl SurfaceId for FlorSurfaceId {}
