use graphics::base::SurfaceId;
#[cfg(feature = "direct2d")]
use graphics::handle::D2DSurfaceId;
#[cfg(feature = "opengl")]
use graphics::handle::GlSurfaceId;

#[derive(Debug, Clone)]
pub enum FlorSurfaceId {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DSurfaceId,
        #[cfg(feature = "opengl")] GlSurfaceId,
    ),
}
impl SurfaceId for FlorSurfaceId {}
