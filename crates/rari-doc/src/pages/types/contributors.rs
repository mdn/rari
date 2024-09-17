use std::path::{Path, PathBuf};
use std::sync::Arc;

use concat_in_place::strcat;
use rari_md::m2h;
use rari_types::error::EnvError;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::globals::contributor_spotlight_root;
use rari_types::locale::Locale;
use rari_types::RariEnv;
use rari_utils::io::read_to_string;
use serde::{Deserialize, Serialize};

use crate::error::DocError;
use crate::pages::page::{Page, PageLike, PageReader};
use crate::utils::split_fm;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Usernames {
    pub github: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct ContributorFrontMatter {
    pub contributor_name: String,
    pub folder_name: String,
    pub is_featured: bool,
    pub img_alt: String,
    pub usernames: Usernames,
    pub quote: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ContributorMeta {
    pub slug: String,
    pub title: String,
    pub contributor_name: String,
    pub folder_name: String,
    pub is_featured: bool,
    pub img: String,
    pub img_alt: String,
    pub usernames: Usernames,
    pub quote: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ContributorBuildMeta {
    pub locale: Locale,
    pub slug: String,
    pub title: String,
    pub url: String,
    pub contributor_name: String,
    pub folder_name: String,
    pub is_featured: bool,
    pub img: String,
    pub img_alt: String,
    pub usernames: Usernames,
    pub quote: String,
    pub path: PathBuf,
    pub full_path: PathBuf,
}

impl From<&ContributorBuildMeta> for ContributorMeta {
    fn from(value: &ContributorBuildMeta) -> Self {
        let ContributorBuildMeta {
            slug,
            title,
            contributor_name,
            folder_name,
            is_featured,
            img,
            img_alt,
            usernames,
            quote,
            ..
        } = value;
        ContributorMeta {
            slug: slug.clone(),
            title: title.clone(),
            contributor_name: contributor_name.clone(),
            folder_name: folder_name.clone(),
            is_featured: *is_featured,
            img: img.clone(),
            img_alt: img_alt.clone(),
            usernames: usernames.clone(),
            quote: quote.clone(),
        }
    }
}

impl ContributorBuildMeta {
    pub fn from_fm(
        fm: ContributorFrontMatter,
        full_path: impl Into<PathBuf>,
        locale: Locale,
    ) -> Result<Self, DocError> {
        let full_path = full_path.into();
        let path = full_path
            .strip_prefix(
                contributor_spotlight_root().ok_or(EnvError::NoContributorSpotlightRoot)?,
            )?
            .into();
        let ContributorFrontMatter {
            contributor_name,
            folder_name,
            is_featured,
            img_alt,
            usernames,
            quote,
        } = fm;
        let slug = strcat!("spotlight/" folder_name.as_str());
        Ok(Self {
            url: strcat!("/" locale.as_url_str() "/community/" slug.as_str()),
            locale,
            title: strcat!("Contributor Spotlight - " contributor_name.as_str() " | MDN"),
            slug,
            contributor_name,
            folder_name,
            is_featured,
            img: "profile-image.jpg".to_string(),
            img_alt,
            usernames,
            quote,
            full_path,
            path,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ContributorSpotlight {
    pub meta: ContributorBuildMeta,
    raw: String,
    content_start: usize,
}

impl ContributorSpotlight {
    pub fn as_locale(&self, locale: Locale) -> Self {
        let Self {
            mut meta,
            raw,
            content_start,
        } = self.clone();
        meta.locale = locale;
        meta.url = strcat!("/" locale.as_url_str() "/community/" meta.slug.as_str());
        Self {
            meta,
            raw,
            content_start,
        }
    }
}

impl PageReader for ContributorSpotlight {
    fn read(path: impl Into<PathBuf>, locale: Option<Locale>) -> Result<Page, DocError> {
        read_contributor_spotlight(path, locale.unwrap_or_default())
            .map(Arc::new)
            .map(Page::ContributorSpotlight)
    }
}

impl PageLike for ContributorSpotlight {
    fn url(&self) -> &str {
        &self.meta.url
    }

    fn slug(&self) -> &str {
        &self.meta.slug
    }

    fn title(&self) -> &str {
        &self.meta.title
    }

    fn short_title(&self) -> Option<&str> {
        None
    }

    fn locale(&self) -> Locale {
        Locale::EnUs
    }

    fn content(&self) -> &str {
        &self.raw[self.content_start..]
    }

    fn rari_env(&self) -> Option<RariEnv<'_>> {
        Some(RariEnv {
            url: &self.meta.url,
            locale: self.meta.locale,
            title: &self.meta.title,
            tags: &[],
            browser_compat: &[],
            spec_urls: &[],
            page_type: PageType::BlogPost,
            slug: &self.meta.slug,
        })
    }

    fn render(&self) -> Result<String, DocError> {
        Ok(m2h(self.content(), Locale::EnUs)?)
    }

    fn title_suffix(&self) -> Option<&str> {
        Some("MDN Blog")
    }

    fn page_type(&self) -> PageType {
        PageType::BlogPost
    }

    fn status(&self) -> &[FeatureStatus] {
        &[]
    }

    fn full_path(&self) -> &Path {
        &self.meta.full_path
    }

    fn path(&self) -> &Path {
        &self.meta.path
    }

    fn base_slug(&self) -> &str {
        "/en-US/"
    }

    fn trailing_slash(&self) -> bool {
        true
    }
}

fn read_contributor_spotlight(
    path: impl Into<PathBuf>,
    locale: Locale,
) -> Result<ContributorSpotlight, DocError> {
    let full_path = path.into();
    let raw = read_to_string(&full_path)?;
    let (fm, content_start) = split_fm(&raw);
    let fm = fm.ok_or(DocError::NoFrontmatter)?;
    let fm: ContributorFrontMatter = serde_yaml::from_str(fm)?;

    Ok(ContributorSpotlight {
        meta: ContributorBuildMeta::from_fm(fm, full_path, locale)?,
        raw,
        content_start,
    })
}

pub fn contributor_spotlight_from_url(url: &str, locale: Locale) -> Option<Page> {
    if let Some(folder_name) = url.split('/').nth(4) {
        contributor_spotlight_root()
            .map(|root| root.join(folder_name).join("index.md"))
            .and_then(|path| {
                read_contributor_spotlight(path, locale)
                    .map_err(|e| {
                        tracing::error!("{e}");
                        e
                    })
                    .ok()
            })
            .map(|page| Page::ContributorSpotlight(Arc::new(page)))
    } else {
        None
    }
}
