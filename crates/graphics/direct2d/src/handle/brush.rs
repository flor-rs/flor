use flor_graphics_base::BrushHandle;
use windows::Win32::Graphics::Direct2D::ID2D1Brush;

#[derive(Debug, Clone)]
pub struct D2DBrushHandle {
    raw: ID2D1Brush,
}
impl BrushHandle for D2DBrushHandle {}

impl D2DBrushHandle {
    pub fn new(raw: ID2D1Brush) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &ID2D1Brush {
        &self.raw
    }
}
