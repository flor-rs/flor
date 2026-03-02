use flor_base::graphics::SurfaceId;
use glow::HasContext;

#[derive(Debug, Clone)]
pub struct GlSurfaceId {
    pub width: u32,
    pub height: u32,
    pub texture: glow::Texture,
    pub fbo: glow::Framebuffer,
}

impl SurfaceId for GlSurfaceId {}
