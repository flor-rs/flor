pub mod brush_handle;
pub mod error;
pub mod image_handle;
pub mod load_resource;
pub mod renderer;
pub mod surface_id;
#[cfg(feature = "svg")]
pub mod svg_handle;
pub mod text_format_handle;
pub mod text_layout;

// #[cfg(all(
//     not(any(feature = "direct2d", feature = "dx11")), // GPU 后端
//     not(any(feature = "gdi", feature = "skia_cpu")) // CPU 后端
// ))]
// compile_error!("至少需要启用一个渲染器特性（GPU 或 CPU）");
//
// // ==========================
// // 2️⃣ GPU 后端不能同时启用多个
// // ==========================
// #[cfg(all(feature = "direct2d", feature = "dx11"))]
// compile_error!("同类型 GPU 后端不能同时启用 multiple features");
//
// // ==========================
// // 3️⃣ CPU 后端不能同时启用多个
// // ==========================
// #[cfg(all(feature = "gdi", feature = "skia_cpu"))]
// compile_error!("同类型 CPU 后端不能同时启用 multiple features");
#[cfg(feature = "svg")]
pub use svg_handle::*;

pub use {
    brush_handle::*, error::*, image_handle::*, load_resource::*, renderer::*, surface_id::*,
    text_format_handle::*, text_layout::*,
};
