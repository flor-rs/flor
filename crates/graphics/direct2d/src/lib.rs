// 只能在 Windows 平台启用 direct2d 后端
#[cfg(not(target_os = "windows"))]
compile_error!("The 'direct2d' feature is only supported on Windows platforms.");

pub mod error;

pub mod base {
    pub use flor_base::graphics::*;
}

pub mod encode;
pub mod handle;
pub mod to_d2d;

#[cfg(feature = "memory-font")]
mod memory_font;

mod renderer;
pub use renderer::*;

mod render_factory;
