use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use constcat::concat;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::globals::content_translated_root;
use rari_types::locale::Locale;
use rari_types::RariEnv;
use rari_utils::concat_strs;

use super::spa_homepage::{
    featured_articles, featured_contributor, latest_news, recent_contributions,
};
use crate::cached_readers::{blog_files, generic_content_config, BasicSPA, BuildSPA, SPAData};
use crate::error::DocError;
use crate::helpers::title::page_title;
use crate::pages::json::{
    BlogIndex, BuiltPage, ItemContainer, JsonBlogPostDoc, JsonBlogPostPage, JsonHomePage,
    JsonHomePageSPAHyData, JsonSPAPage,
};
use crate::pages::page::{Page, PageLike, PageReader};
use crate::pages::types::blog::BlogMeta;

#[derive(Debug, Clone)]
pub struct SPA {
    pub page_title: &'static str,
    pub slug: &'static str,
    pub url: String,
    pub locale: Locale,
    pub page_type: PageType,
    pub data: SPAData,
    pub base_slug: Cow<'static, str>,
    pub page_description: Option<&'static str>,
}
impl SPA {
    pub fn from_url(url: &str) -> Option<Page> {
        match url {
            "/en-US/blog/" | "/en-us/blog/" => SPA::from_slug("blog", Locale::EnUs),
            _ => None,
        }
    }

    pub fn from_slug(slug: &str, locale: Locale) -> Option<Page> {
        BASIC_SPAS.get(slug).and_then(|build_spa| {
            if build_spa.en_us_only && locale != Locale::EnUs {
                None
            } else {
                Some(Page::SPA(Arc::new(SPA {
                    page_title: &build_spa.page_title,
                    slug: &build_spa.slug,
                    url: concat_strs!(
                        "/",
                        locale.as_url_str(),
                        "/",
                        &build_spa.slug,
                        if build_spa.trailing_slash && !build_spa.slug.is_empty() {
                            "/"
                        } else {
                            ""
                        }
                    ),
                    locale,
                    page_type: PageType::SPA,
                    data: build_spa.data,
                    base_slug: Cow::Owned(concat_strs!("/", locale.as_url_str(), "/")),
                    page_description: build_spa.page_description.as_deref(),
                })))
            }
        })
    }

    pub fn is_spa(slug: &str, locale: Locale) -> bool {
        BASIC_SPAS
            .get(slug)
            .map(|build_spa| locale == Default::default() || !build_spa.en_us_only)
            .unwrap_or_default()
    }

    pub fn all() -> Vec<(String, Locale)> {
        BASIC_SPAS
            .iter()
            .flat_map(|(slug, build_spa)| {
                if build_spa.en_us_only || content_translated_root().is_none() {
                    vec![(slug.clone(), Locale::EnUs)]
                } else {
                    Locale::for_generic_and_spas()
                        .iter()
                        .map(|locale| (slug.clone(), *locale))
                        .collect()
                }
            })
            .collect()
    }

    pub fn as_built_doc(&self) -> Result<BuiltPage, DocError> {
        match &self.data {
            SPAData::BlogIndex => Ok(BuiltPage::BlogPost(Box::new(JsonBlogPostPage {
                doc: JsonBlogPostDoc {
                    title: self.title().to_string(),
                    mdn_url: self.url().to_owned(),
                    native: self.locale().into(),
                    page_title: page_title(self, true)?,
                    locale: self.locale(),
                    ..Default::default()
                },
                url: self.url().to_owned(),
                locale: self.locale(),
                blog_meta: None,
                hy_data: Some(BlogIndex {
                    posts: blog_files()
                        .sorted_meta
                        .iter()
                        .rev()
                        .map(BlogMeta::from)
                        .map(|mut m| {
                            m.links = Default::default();
                            m
                        })
                        .collect(),
                }),
                page_title: self.title().to_owned(),
                ..Default::default()
            }))),
            SPAData::BasicSPA(basic_spa) => Ok(BuiltPage::SPA(Box::new(JsonSPAPage {
                slug: self.slug,
                page_title: self.page_title,
                page_description: self.page_description,
                only_follow: basic_spa.only_follow,
                no_indexing: basic_spa.no_indexing,
                page_not_found: false,
                url: concat_strs!(self.base_slug.as_ref(), self.slug),
            }))),
            SPAData::NotFound => Ok(BuiltPage::SPA(Box::new(JsonSPAPage {
                slug: self.slug,
                page_title: self.page_title,
                page_description: self.page_description,
                only_follow: false,
                no_indexing: true,
                page_not_found: true,
                url: concat_strs!(self.base_slug.as_ref(), self.slug),
            }))),
            SPAData::HomePage => Ok(BuiltPage::Home(Box::new(JsonHomePage {
                url: concat_strs!("/", self.locale().as_url_str(), "/", self.slug),
                page_title: self.page_title,
                hy_data: JsonHomePageSPAHyData {
                    page_description: self.page_description,
                    featured_articles: featured_articles(
                        &[
                            "/en-US/blog/mdn-scrimba-partnership/",
                            "/en-US/blog/learn-javascript-console-methods/",
                            "/en-US/blog/introduction-to-web-sustainability/",
                            "/en-US/docs/Web/API/CSS_Custom_Highlight_API",
                        ],
                        self.locale,
                    )?,
                    featured_contributor: featured_contributor(self.locale)?,
                    latest_news: ItemContainer {
                        items: latest_news(&[
                            "/en-US/blog/mdn-scrimba-partnership/",
                            "/en-US/blog/mdn-http-observatory-launch/",
                            "/en-US/blog/mdn-curriculum-launch/",
                            "/en-US/blog/baseline-evolution-on-mdn/",
                        ])?,
                    },
                    recent_contributions: ItemContainer {
                        items: recent_contributions()?,
                    },
                },
            }))),
        }
    }
}

impl PageReader<Page> for SPA {
    fn read(_: impl Into<PathBuf>, _: Option<Locale>) -> Result<Page, DocError> {
        todo!()
    }
}

impl PageLike for SPA {
    fn url(&self) -> &str {
        &self.url
    }

    fn slug(&self) -> &str {
        self.slug
    }

    fn title(&self) -> &str {
        self.page_title
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
        todo!()
    }

    fn title_suffix(&self) -> Option<&str> {
        Some("MDN")
    }

    fn page_type(&self) -> PageType {
        self.page_type
    }

    fn status(&self) -> &[FeatureStatus] {
        &[]
    }

    fn full_path(&self) -> &Path {
        Path::new("")
    }

    fn path(&self) -> &Path {
        Path::new("")
    }

    fn base_slug(&self) -> &str {
        &self.base_slug
    }

    fn trailing_slash(&self) -> bool {
        self.url().ends_with('/')
    }

    fn fm_offset(&self) -> usize {
        0
    }
}

const MDN_PLUS_TITLE: &str = "MDN Plus";
const OBSERVATORY_TITLE_FULL: &str = "HTTP Observatory | MDN";

const OBSERVATORY_DESCRIPTION: &str =
"Test your site’s HTTP headers, including CSP and HSTS, to find security problems and get actionable recommendations to make your website more secure. Test other websites to see how you compare.";

static BASIC_SPAS: LazyLock<HashMap<String, BuildSPA>> = LazyLock::new(|| {
    generic_content_config()
        .spas
        .clone()
        .into_iter()
        .chain(
            [
                (
                    "",
                    BuildSPA {
                        slug: Cow::Borrowed(""),
                        page_title: Cow::Borrowed("MDN Web Docs"),
                        trailing_slash: true,
                        data: SPAData::HomePage,
                        ..Default::default()
                    },
                ),
                (
                    "404",
                    BuildSPA {
                        slug: Cow::Borrowed("404"),
                        page_title: Cow::Borrowed("404"),
                        en_us_only: true,
                        data: SPAData::NotFound,
                        ..Default::default()
                    },
                ),
                (
                    "blog",
                    BuildSPA {
                        slug: Cow::Borrowed("blog"),
                        page_title: Cow::Borrowed("MDN Blog"),
                        trailing_slash: true,
                        en_us_only: true,
                        data: SPAData::BlogIndex,
                        ..Default::default()
                    },
                ),
                (
                    "play",
                    BuildSPA {
                        slug: Cow::Borrowed("play"),
                        page_title: Cow::Borrowed("Playground | MDN"),
                        ..Default::default()
                    },
                ),
                (
                    "observatory",
                    BuildSPA {
                        slug: Cow::Borrowed("observatory"),
                        page_title: Cow::Borrowed(concat!(
                            "HTTP Header Security Test - ",
                            OBSERVATORY_TITLE_FULL
                        )),
                        page_description: Some(Cow::Borrowed(OBSERVATORY_DESCRIPTION)),
                        ..Default::default()
                    },
                ),
                (
                    "observatory/analyze",
                    BuildSPA {
                        slug: Cow::Borrowed("observatory/analyze"),
                        page_title: Cow::Borrowed(concat!(
                            "Scan results - ",
                            OBSERVATORY_TITLE_FULL
                        )),
                        page_description: Some(Cow::Borrowed(OBSERVATORY_DESCRIPTION)),
                        data: SPAData::BasicSPA(BasicSPA {
                            no_indexing: true,
                            only_follow: false,
                        }),
                        ..Default::default()
                    },
                ),
                (
                    "search",
                    BuildSPA {
                        slug: Cow::Borrowed("search"),
                        page_title: Cow::Borrowed("Search"),
                        data: SPAData::BasicSPA(BasicSPA {
                            only_follow: true,
                            no_indexing: false,
                        }),
                        ..Default::default()
                    },
                ),
                (
                    "plus/ai-help",
                    BuildSPA {
                        slug: Cow::Borrowed("plus/ai-help"),
                        page_title: Cow::Borrowed(concat!("AI Help | ", MDN_PLUS_TITLE)),
                        ..Default::default()
                    },
                ),
                (
                    "plus/collections",
                    BuildSPA {
                        slug: Cow::Borrowed("plus/collections"),
                        page_title: Cow::Borrowed(concat!("Collections | ", MDN_PLUS_TITLE)),
                        data: SPAData::BasicSPA(BasicSPA {
                            no_indexing: true,
                            only_follow: false,
                        }),
                        ..Default::default()
                    },
                ),
                (
                    "plus/collections/frequently_viewed",
                    BuildSPA {
                        slug: Cow::Borrowed("plus/collections/frequently_viewed"),
                        page_title: Cow::Borrowed(concat!(
                            "Frequently viewed articles | ",
                            MDN_PLUS_TITLE
                        )),
                        data: SPAData::BasicSPA(BasicSPA {
                            no_indexing: true,
                            only_follow: false,
                        }),
                        ..Default::default()
                    },
                ),
                (
                    "plus/updates",
                    BuildSPA {
                        slug: Cow::Borrowed("plus/updates"),
                        page_title: Cow::Borrowed(concat!("Updates | ", MDN_PLUS_TITLE)),
                        ..Default::default()
                    },
                ),
                (
                    "plus/settings",
                    BuildSPA {
                        slug: Cow::Borrowed("plus/settings"),
                        page_title: Cow::Borrowed(concat!("Settings | ", MDN_PLUS_TITLE)),
                        data: SPAData::BasicSPA(BasicSPA {
                            no_indexing: true,
                            only_follow: false,
                        }),
                        ..Default::default()
                    },
                ),
                (
                    "newsletter",
                    BuildSPA {
                        slug: Cow::Borrowed("newsletter"),
                        page_title: Cow::Borrowed("Stay Informed with MDN"),
                        ..Default::default()
                    },
                ),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v)),
        )
        .collect()
});

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn print() {
        println!("{}", serde_json::to_string(&*BASIC_SPAS).unwrap())
    }
}
