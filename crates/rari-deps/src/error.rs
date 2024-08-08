use thiserror::Error;

#[derive(Debug, Error)]
pub enum DepsError {
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    RariIoError(#[from] rari_utils::error::RariIoError),
    #[error(transparent)]
    FetchError(#[from] reqwest::Error),
    #[error("no version for webref")]
    WebRefMissingVersionError,
    #[error("no tarball for webref")]
    WebRefMissingTarballError,
}
