#[cfg(target_os = "windows")]
pub mod window;

#[cfg(target_os = "windows")]
pub use window::GlDisplayContext as NativeDisplayContext;
