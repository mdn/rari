use std::borrow::Cow;

use rari_doc::error::{DocError, UrlError};
use rari_types::error::EnvError;
use rari_types::locale::LocaleError;
use rari_utils::error::RariIoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Invalid slug: {0}")]
    InvalidSlug(Cow<'static, str>),
    #[error("Git error: {0}")]
    GitError(String),

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
    RariIoError(#[from] RariIoError),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid Redirection: {0}")]
    InvalidRedirectionEntry(String),
    #[error("Error reading redirects file: {0}")]
    ReadRedirectsError(String),
    #[error("Error writing redirects file: {0}")]
    WriteRedirectsError(String),
    #[error("Invalid 'from' URL for redirect: {0}")]
    InvalidRedirectFromURL(String),
    #[error("Invalid 'to' URL for redirect: {0}")]
    InvalidRedirectToURL(String),
    #[error(transparent)]
    RedirectError(#[from] RedirectError),

    #[error("Unknonwn error")]
    Unknown(&'static str),
}

#[derive(Debug, Clone, Error)]
pub enum RedirectError {
    #[error("RedirectError: {0}")]
    Cycle(String),
    #[error("No cased version {0}")]
    NoCased(String),
}
