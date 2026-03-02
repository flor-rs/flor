use crate::renderer::context::GlContext;
use flor_base::graphics::SurfaceId;
use glow::HasContext;
use std::sync::Arc;

#[derive(Debug)]
pub struct SurfaceInner {
    pub gl_context: Arc<GlContext>,
    pub texture: glow::Texture,
    pub fbo: glow::Framebuffer,
    pub rbo: Option<glow::Renderbuffer>,
}

impl Drop for SurfaceInner {
    fn drop(&mut self) {
        unsafe {
            self.gl_context.delete_texture(self.texture);
            self.gl_context.delete_framebuffer(self.fbo);
            if let Some(rbo) = self.rbo {
                self.gl_context.delete_renderbuffer(rbo);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlSurfaceId {
    pub width: u32,
    pub height: u32,
    pub inner: Arc<SurfaceInner>,
}

impl SurfaceId for GlSurfaceId {}
