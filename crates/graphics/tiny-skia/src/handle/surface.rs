use flor_base::graphics::SurfaceId;
use slotmap::new_key_type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TinySkiaSurfaceId {
    pub id: SurfaceSlotId,
}
impl SurfaceId for TinySkiaSurfaceId {}

new_key_type! {
    pub struct SurfaceSlotId;
}
