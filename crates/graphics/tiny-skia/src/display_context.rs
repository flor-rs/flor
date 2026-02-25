use crate::TinySkiaConfig;

mod platform;
use crate::error::TinySkiaError;
pub use platform::*;

pub trait DisplayContext: Send + Sync {
    type HWND;
    fn create(h_wnd: Self::HWND, config: TinySkiaConfig) -> Result<Self, TinySkiaError>
    where
        Self: Sized;
    fn present(&mut self, width: u32, height: u32, pixels: &[u8]) -> Result<(), TinySkiaError>;
    fn wait_v_sync(&self);
}
