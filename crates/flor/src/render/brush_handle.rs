use flor_graphics_base::BrushHandle;
use graphics::handle::D2DBrushHandle;

#[derive(Debug)]
pub enum FlorBrushHandle {
    #[cfg(feature = "direct2d")]
    D2DBrushHandle(D2DBrushHandle),
}

impl BrushHandle for FlorBrushHandle {}
