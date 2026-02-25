use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("provider error: {0}")]
    Provider(#[from] crate::provider::ProviderError),

    #[error("output error: {0}")]
    Output(String),

    #[error("application not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
