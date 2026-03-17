#[cfg(feature = "graphics")]
pub mod graphics;
#[cfg(feature = "platform")]
pub mod platform;
pub mod types;

// 1. 只有在【没开后端】且【没开跳过开关】时，才报“缺少后端”的错误
#[cfg(all(
    not(any(feature = "gpu-render-backend", feature = "cpu-render-backend")),
    not(feature = "no-check-backend")
))]
compile_error!("You need to enable at least one rendering backend or enable 'no-check-backend'.");

// 2. 无论有没有 no-check-backend，只要同时开了两个 GPU 后端，就必须报错
// 这样可以保证即便中间件跳过了检查，最终产物如果配置冲突依然能报错
#[cfg(all(feature = "direct2d", feature = "opengl"))]
compile_error!("Only one GPU rendering backend can be enabled at most.");
