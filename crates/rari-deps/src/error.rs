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
    #[error(transparent)]
    HeaderError(#[from] reqwest::header::ToStrError),
    #[error("no version for webref")]
    WebRefMissingVersionError,
    #[error("no tarball for webref")]
    WebRefMissingTarballError,
    #[error("Invalid github version")]
    InvalidGitHubVersion,
    #[error("Invalid github version")]
    VersionNotFound,
}
