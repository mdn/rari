use std::path::{Path, PathBuf};

use rari_types::RariEnv;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::pages::page::PageLike;
use crate::pages::types::utils::FmTempl;

/// Minimal [`PageLike`] implementation for unit tests.
///
/// Only `url`, `path`/`full_path`, and `locale` carry real data;
/// every other method returns a harmless empty/default value.
pub struct TestPage {
    pub path: PathBuf,
    pub url: String,
    pub locale: Locale,
}

impl Default for TestPage {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            url: String::new(),
            locale: Locale::EnUs,
        }
    }
}

impl PageLike for TestPage {
    fn url(&self) -> &str {
        &self.url
    }

    fn slug(&self) -> &str {
        ""
    }

    fn title(&self) -> &str {
        ""
    }

    fn short_title(&self) -> Option<&str> {
        None
    }

    fn locale(&self) -> Locale {
        self.locale
    }

    fn content(&self) -> &str {
        ""
    }

    fn rari_env(&self) -> Option<RariEnv<'_>> {
        None
    }

    fn render(&self) -> Result<String, DocError> {
        Ok(String::new())
    }

    fn title_suffix(&self) -> Option<&str> {
        None
    }

    fn page_type(&self) -> PageType {
        PageType::None
    }

    fn status(&self) -> &[FeatureStatus] {
        &[]
    }

    fn full_path(&self) -> &Path {
        &self.path
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn base_slug(&self) -> &str {
        ""
    }

    fn trailing_slash(&self) -> bool {
        false
    }

    fn fm_offset(&self) -> usize {
        0
    }

    fn raw_content(&self) -> &str {
        ""
    }

    fn banners(&self) -> Option<&[FmTempl]> {
        None
    }
}
