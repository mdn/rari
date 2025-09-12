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
    #[error("Invalid url: {0}")]
    UrlError(#[from] url::ParseError),
    #[error("Version key not found in package.json")]
    PackageVersionNotFound,
    #[error("Version from package.json could not be parsed")]
    PackageVersionParseError,
}
