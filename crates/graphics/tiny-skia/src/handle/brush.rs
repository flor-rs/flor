use flor_base::graphics::{BrushHandle, Gradient as FlorGradient};
use slotmap::new_key_type;

#[derive(Debug, Clone)]
pub enum TinySkiaBrushHandle {
    Solid(tiny_skia::Color),
    Gradient { gradient: FlorGradient },
}

impl BrushHandle for TinySkiaBrushHandle {}

new_key_type! {
    pub struct BrushSlotId;
}
