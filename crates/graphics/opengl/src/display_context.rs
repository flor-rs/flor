mod platform;
use crate::error::GlError;
use crate::GlConfig;
pub use platform::*;

pub trait DisplayContext: Send + Sync {
    type HWND;
    fn create(h_wnd: Self::HWND, config: GlConfig) -> Result<Self, GlError>
    where
        Self: Sized;
    fn get_gl_context(&self) -> glow::Context;
    fn set_v_sync(&self, v_sync: bool);

    fn present(&self);
}
