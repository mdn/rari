use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] rari_utils::error::RariIoError),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}
