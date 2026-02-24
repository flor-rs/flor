use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlRendererError {
    #[error("window core error: `{0}`")]
    WindowCore(#[from] windows::core::Error),
}
