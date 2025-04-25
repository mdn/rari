use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Copy, Error)]
pub enum EnvError {
    #[error("CONTENT_ROOT must be set")]
    NoContent,
    #[error("CONTENT_TRANSLATED_ROOT must be set")]
    NoTranslatedContent,
    #[error("BUILD_OUT_ROOT must be set")]
    NoBuildOut,
    #[error("CONTRIBUTOR_SPOTLIGHT_ROOT must be set")]
    NoContributorSpotlightRoot,
}
