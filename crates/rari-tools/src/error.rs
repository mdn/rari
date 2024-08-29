use rari_doc::error::DocError;
use rari_types::locale::LocaleError;
// use rari_types::locale::LocaleError;
// use rari_types::ArgError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("invalid slug: {0}")]
    InvalidSlug(String),
    // #[error("invalid locale: {0}")]
    // InvalidLocale(String),
    #[error(transparent)]
    LocaleError(#[from] LocaleError),
    #[error(transparent)]
    DocError(#[from] DocError),

    #[error("Unknonwn error")]
    Unknown(String),
}
