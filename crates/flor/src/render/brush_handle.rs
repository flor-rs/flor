use flor_base::graphics::BrushHandle;
use graphics::handle::D2DBrushHandle;

#[derive(Debug, Clone)]
pub enum FlorBrushHandle {
    #[cfg(feature = "direct2d")]
    D2DBrushHandle(D2DBrushHandle),
}

impl BrushHandle for FlorBrushHandle {}
