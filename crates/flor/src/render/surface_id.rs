use flor_graphics_base::SurfaceId;
use graphics::handle::D2DSurfaceId;

#[derive(Debug,Clone)]
pub enum FlorSurfaceId {
    #[cfg(feature = "direct2d")]
    D2DSurfaceId(D2DSurfaceId),
}
impl SurfaceId for FlorSurfaceId {}
