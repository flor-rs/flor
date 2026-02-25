use flor_base::graphics::SvgHandle;
use slotmap::new_key_type;
#[derive(Clone)]
pub struct SvgCache {
    pub svg_pixmap: Option<(tiny_skia::Pixmap, (f32, f32))>, // (Pixmap, (scale_x, scale_y))
    pub shadow_pixmap: Option<(tiny_skia::Pixmap, (f32, f32), f32)>, // (Pixmap, (scale_x, scale_y), blur_radius)
}

#[derive(Clone)]
pub struct TinySkiaSvgHandle {
    pub width: u32,
    pub height: u32,
    pub tree: std::sync::Arc<usvg::Tree>,
    pub cache: std::sync::Arc<parking_lot::RwLock<SvgCache>>,
}

impl std::fmt::Debug for TinySkiaSvgHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TinySkiaSvgHandle")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl SvgHandle for TinySkiaSvgHandle {}

new_key_type! {
    pub struct SvgSlotId;
}
