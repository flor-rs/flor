use flor_base::graphics::SvgHandle;
use glow::NativeTexture;
use parking_lot::Mutex;
use std::sync::Arc;
use usvg::Tree;

#[derive(Debug)]
pub struct GlSvgHandle {
    pub cache: Mutex<Option<(u32, u32, NativeTexture)>>,
    pub tree: Arc<Tree>,
}

impl Clone for GlSvgHandle {
    fn clone(&self) -> Self {
        Self {
            cache: Mutex::new(None),
            tree: Arc::clone(&self.tree),
        }
    }
}

impl SvgHandle for GlSvgHandle {}
