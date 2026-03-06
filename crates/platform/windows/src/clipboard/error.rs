use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("clipboard is empty")]
    ClipboardIsEmpty,
    #[error("invalid encoding")]
    System(#[from] windows::core::Error),
}
