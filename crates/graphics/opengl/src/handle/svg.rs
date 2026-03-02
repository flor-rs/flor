use crate::renderer::context::GlContext;
use flor_base::graphics::SvgHandle;
use glow::NativeTexture;
use parking_lot::Mutex;
use std::sync::Arc;
use usvg::Tree;

pub struct SvgCacheInner {
    pub gl_context: Arc<GlContext>,
    pub texture: Mutex<Option<(u32, u32, NativeTexture)>>,
}

impl Drop for SvgCacheInner {
    fn drop(&mut self) {
        if let Some((_, _, tex)) = self.texture.lock().take() {
            unsafe {
                use glow::HasContext;
                self.gl_context.delete_texture(tex);
            }
        }
    }
}

impl std::fmt::Debug for SvgCacheInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SvgCacheInner").finish()
    }
}

#[derive(Debug, Clone)]
pub struct GlSvgHandle {
    pub cache: Arc<SvgCacheInner>,
    pub tree: Arc<Tree>,
}

impl SvgHandle for GlSvgHandle {}
