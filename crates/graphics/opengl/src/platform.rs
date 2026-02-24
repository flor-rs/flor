#[cfg(target_os = "windows")]
mod window;

#[cfg(target_os = "windows")]
pub use window::*;
