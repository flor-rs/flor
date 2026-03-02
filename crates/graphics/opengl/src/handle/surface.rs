use flor_base::graphics::SurfaceId;

#[derive(Debug, Clone)]
pub struct GlSurfaceId {
    pub width: u32,
    pub height: u32,
    pub texture: glow::Texture,
    pub fbo: glow::Framebuffer,
    pub rbo: Option<glow::Renderbuffer>,
}

impl SurfaceId for GlSurfaceId {}
