use rari_doc::error::{DocError, UrlError};
use rari_types::{error::EnvError, locale::LocaleError};
// use rari_types::locale::LocaleError;
// use rari_types::ArgError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("invalid slug: {0}")]
    InvalidSlug(String),

    #[error(transparent)]
    LocaleError(#[from] LocaleError),
    #[error(transparent)]
    DocError(#[from] DocError),
    #[error(transparent)]
    EnvError(#[from] EnvError),
    #[error(transparent)]
    UrlError(#[from] UrlError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error("Unknonwn error")]
    Unknown(String),
}
