use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlError {
    #[error("window core error: `{0}`")]
    WindowCore(#[from] windows::core::Error),
    #[error("image error: `{0}`")]
    ImageError(#[from] image::ImageError),
    #[error("custom error: `{0}`")]
    CustomError(String),
}

impl From<String> for GlError {
    fn from(value: String) -> Self {
        Self::CustomError(value)
    }
}
