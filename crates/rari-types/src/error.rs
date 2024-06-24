use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Copy, Error)]
pub enum EnvError {
    #[error("CONTENT_ROOT must be set")]
    NoContent,
    #[error("TRANSLATED_CONTENT_ROOT must be set")]
    NoTranslatedContent,
    #[error("BUILD_OUT_ROOT must be set")]
    NoBuildOut,
}
