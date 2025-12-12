use std::fmt::Debug;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SurfaceId resource not find: `{0}`")]
    SurfaceIdHandleNotFound(usize),

    #[error("SvgHandle resource not find: `{0}`")]
    SvgHandleNotFound(usize),

    #[error("ImageHandle resource not find: `{0}`")]
    ImageHandleNotFound(usize),

    #[error("Image frame not find: `{0}`")]
    ImageFrameNotFound(usize),

    #[error("Image frame count `{0}` does not match delay count `{1}`")]
    ImageFrameDelayMismatch(usize, usize),

    #[error("BrushHandle resource not find: `{0}`")]
    BrushHandleNotFound(usize),

    #[error("TextFormatHandle resource not find: `{0}`")]
    TextFormatHandleNotFound(usize),

    #[error("color parse error: `{0}`")]
    ColorParseColor(#[from] crate::ColorParseError),
}
