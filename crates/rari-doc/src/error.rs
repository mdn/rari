use std::path::{PathBuf, StripPrefixError};

use css_syntax::error::SyntaxError;
use rari_md::error::MarkdownError;
use rari_types::error::EnvError;
use rari_types::locale::LocaleError;
use rari_types::ArgError;
use thiserror::Error;

use crate::helpers::l10n::L10nError;
use crate::pages::page::PageCategory;

/// Represents various errors that can occur when processing pages.
///
/// Each variant corresponds to a specific error type and includes relevant
/// error messages and associated data.
#[derive(Debug, Error)]
pub enum DocError {
    #[error("Not a Doc")]
    NotADoc,
    #[error("Cannot parse templ index")]
    TemplIndexParseError(#[from] std::num::ParseIntError),
    #[error("Invalid templ index {0}")]
    InvalidTemplIndex(usize),
    #[error("No parent")]
    NoParent(PathBuf),
    #[error(transparent)]
    NoSuchPrefix(#[from] StripPrefixError),
    #[error("No curriculum root set")]
    NoCurriculumRoot,
    #[error("No generic content root set")]
    NoGenericContentRoot,
    #[error("No contributor spotlights root set")]
    NoContributorSpotlightRoot,
    #[error("No generic content config found")]
    NoGenericContentConfig,
    #[error("No H1 found")]
    NoH1,
    #[error(transparent)]
    WalkError(#[from] ignore::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error("Page not found (static cache): {0}")]
    NotFoundInStaticCache(String),
    #[error("File cache broken")]
    FileCacheBroken,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Error parsing frontmatter: {0}")]
    FMError(#[from] yaml_rust::scanner::ScanError),
    #[error("Missing frontmatter")]
    NoFrontmatter,
    #[error("Invalid frontmatter: {0}")]
    InvalidFrontmatter(#[from] serde_yaml_ng::Error),
    #[error("Invalid frontmatter to format: {0}")]
    InvalidFrontmatterToFmt(#[from] yaml_parser::SyntaxError),
    #[error(transparent)]
    EnvError(#[from] EnvError),
    #[error(transparent)]
    UrlError(#[from] UrlError),
    #[error(transparent)]
    MarkdownError(#[from] MarkdownError),
    #[error(transparent)]
    LocaleError(#[from] LocaleError),
    #[error("failed to convert bytes: {0}")]
    StrUtf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    LolError(#[from] lol_html::errors::RewritingError),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Link to redirect: {from} -> {to}")]
    RedirectedLink { from: String, to: String },
    #[error("Unknown macro: {0}")]
    UnknownMacro(String),
    #[error("CSS Page type required")]
    CssPageTypeRequired,
    #[error(transparent)]
    ArgError(#[from] ArgError),
    #[error("pest error: {0}")]
    PestError(String),
    #[error("failed to de/serialize")]
    SerializationError,
    #[error(transparent)]
    CssSyntaxError(#[from] SyntaxError),
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
    #[error("invalid templ: {0}")]
    InvalidTempl(String),
    #[error("doc not found {0}")]
    DocNotFound(PathBuf),
    #[error("page({1:?}) not found {0}")]
    PageNotFound(String, PageCategory),
    #[error("no blog root")]
    NoBlogRoot,
    #[error(transparent)]
    L10nError(#[from] L10nError),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Missing CSS l10n in mdn/data")]
    MissingCSSL10n,
    #[error("At rule was empty")]
    MustHaveAtRule,
    #[error("Invalid slug for templ/sidebar: {0}")]
    InvalidSlugForX(String),
    #[error("Invalid group for templ/sidebar: {0}")]
    InvalidGroupForX(String),
    #[error(transparent)]
    RariIoError(#[from] rari_utils::error::RariIoError),
    #[error("Slug required for SidebarEntry")]
    SlugRequiredForSidebarEntry,
    #[error("Invalid sidebar entry")]
    InvalidSidebarEntry,
}

/// Represents various errors that can occur while processing URLs.
#[derive(Debug, Error)]
pub enum UrlError {
    #[error("invalid url")]
    InvalidUrl,
    #[error(transparent)]
    LocaleError(#[from] LocaleError),
    #[error(transparent)]
    EnvError(#[from] EnvError),
}

/// Represents various errors that can occur while handling file operations.
#[derive(Debug, Error)]
pub enum FileError {
    #[error("not a subpath")]
    NoSubPath(#[from] StripPrefixError),
}
