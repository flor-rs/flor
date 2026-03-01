use flor_base::graphics::{BrushHandle, Gradient};
use flor_base::types::Color;

#[derive(Debug, Clone)]
pub enum GlBrushHandle {
    Solid(Color),
    Gradient { gradient: Gradient },
}

impl GlBrushHandle {
    pub(crate) fn to_color_array(&self) -> [f32; 4] {
        match self {
            GlBrushHandle::Solid(c) => [
                c.r as f32 / 255.0,
                c.g as f32 / 255.0,
                c.b as f32 / 255.0,
                c.a as f32 / 255.0,
            ],
            GlBrushHandle::Gradient { .. } => [1.0, 1.0, 1.0, 1.0], // 暂时不支持渐变拾色
        }
    }
}

impl BrushHandle for GlBrushHandle {}
