use thiserror::*;

#[derive(Error, Debug)]
pub enum ComponentError {
    // foreign errors
    #[error(transparent)]
    CoreError(#[from] custom_engine_core::errors::CoreError),
}
