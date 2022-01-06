use thiserror::Error;

#[derive(Debug, Error)]
pub enum BitwiseError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Invalid cursor position: {0}")]
    CursorPosition(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}
