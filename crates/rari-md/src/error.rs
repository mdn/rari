use rari_types::ArgError;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum DocError {
    #[error("pest error: {0}")]
    PestError(String),
    #[error("failed to decode ks: {0}")]
    DecodeError(#[from] base64::DecodeError),
    #[error("failed to convert ks: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("failed to de/serialize")]
    SerializationError,
}

#[derive(Debug, Error)]
pub enum RariFError {
    #[error(transparent)]
    DocError(#[from] DocError),
    #[error(transparent)]
    ArgError(#[from] ArgError),
    #[error("macro not implemented")]
    MacroNotImplemented,
    #[error("unknown macro")]
    UnknownMacro,
}

#[derive(Debug, Error)]
pub enum MarkdownError {
    #[error("unable to output html for markdown")]
    HTMLFormatError,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    DocError(#[from] DocError),
}
