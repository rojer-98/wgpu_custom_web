use thiserror::*;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("shader file `{0}` was not found")]
    FileNotFound(String),

    // foreign errors
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    CoreError(#[from] custom_engine_core::errors::CoreError),
}
