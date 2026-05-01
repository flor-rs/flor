mod display_context;
pub mod error;
pub mod handle;
pub mod renderer;
mod shader;

pub use renderer::*;

pub mod text_layout {
    use crate::error::GlError;
    use crate::handle::GlBrushHandle;
    use flor_base::graphics::CosmicTextLayout;

    pub type GlTextLayout = CosmicTextLayout<GlBrushHandle, GlError>;
}
