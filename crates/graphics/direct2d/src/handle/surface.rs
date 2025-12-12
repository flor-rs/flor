use flor_graphics_base::SurfaceId;
use windows::Win32::Graphics::Direct2D::ID2D1BitmapRenderTarget;

#[derive(Debug, Clone)]
pub struct D2DSurfaceId {
    raw: ID2D1BitmapRenderTarget,
}
impl SurfaceId for D2DSurfaceId {}

impl D2DSurfaceId {
    pub fn new(raw: ID2D1BitmapRenderTarget) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &ID2D1BitmapRenderTarget {
        &self.raw
    }
}
