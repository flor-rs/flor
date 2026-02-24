use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlError {
    #[error("window core error: `{0}`")]
    WindowCore(#[from] windows::core::Error),
}
