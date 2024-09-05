use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use concat_in_place::strcat;
use constcat::concat;
use phf::{phf_map, Map};
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::locale::Locale;
use rari_types::RariEnv;
use serde::Serialize;

use crate::cached_readers::blog_files;
use crate::error::DocError;
use crate::pages::json::{BuiltDocy, HyData, JsonBasicSPA, JsonBlogPost, JsonBlogPostDoc};
use crate::pages::page::{Page, PageCategory, PageLike, PageReader};
use crate::pages::title::page_title;
use crate::pages::types::blog::BlogMeta;

#[derive(Debug, Clone, Serialize)]
pub struct BlogIndex {
    pub posts: Vec<BlogMeta>,
}

#[derive(Debug, Clone)]
pub enum SPAData {
    BlogIndex(BlogIndex),
    BasicSPA(BasicSPA),
}

#[derive(Debug, Clone)]
pub struct SPA {
    pub page_title: &'static str,
    pub slug: &'static str,
    pub url: String,
    pub locale: Locale,
    pub page_type: PageType,
    pub typ: SPAData,
    pub base_slug: Cow<'static, str>,
}
impl SPA {
    pub fn from_url(url: &str) -> Option<Page> {
        match url {
            "/en-US/blog/" | "/en-us/blog/" => SPA::from_slug("blog", Locale::EnUs),
            _ => None,
        }
    }
    pub fn from_slug(slug: &str, locale: Locale) -> Option<Page> {
        if let Some(basic_spa) = BASIC_SPAS.get(slug) {
            return Some(Page::SPA(Arc::new(SPA {
                page_title: basic_spa.page_title,
                slug: basic_spa.slug,
                url: strcat!("/" locale.as_url_str() basic_spa.slug),
                locale,
                page_type: PageType::SPA,
                typ: SPAData::BasicSPA(*basic_spa),
                base_slug: Cow::Owned(strcat!("/" locale.as_url_str() "/")),
            })));
        }
        match (slug, locale) {
            ("blog" | "blog/", Locale::EnUs) => Some(Page::SPA(Arc::new(SPA {
                page_title: "MDN Blog",
                slug: "blog",
                url: "/en-US/blog/".to_string(),
                locale: Locale::EnUs,
                page_type: PageType::SPA,
                typ: SPAData::BlogIndex(BlogIndex {
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
                base_slug: Cow::Borrowed(concat!("/", Locale::EnUs.as_url_str(), "/")),
            }))),
            _ => None,
        }
    }

    pub fn is_spa(slug: &str, locale: Locale) -> bool {
        BASIC_SPAS.contains_key(slug) || matches!((slug, locale), ("blog" | "blog/", Locale::EnUs))
    }

    pub fn as_built_doc(&self) -> Result<BuiltDocy, DocError> {
        match &self.typ {
            SPAData::BlogIndex(b) => Ok(BuiltDocy::BlogPost(Box::new(JsonBlogPost {
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
                hy_data: Some(HyData::BlogIndex(b.clone())),
                page_title: self.title().to_owned(),
                ..Default::default()
            }))),
            SPAData::BasicSPA(basic_spa) => Ok(BuiltDocy::BasicSPA(Box::new(JsonBasicSPA {
                slug: self.slug,
                page_title: self.page_title,
                page_description: basic_spa.page_description,
                only_follow: basic_spa.only_follow,
                no_indexing: basic_spa.no_indexing,
                url: strcat!(self.base_slug.as_ref() self.slug),
            }))),
        }
    }
}

impl PageReader for SPA {
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
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BasicSPA {
    pub slug: &'static str,
    pub page_title: &'static str,
    pub page_description: &'static str,
    pub only_follow: bool,
    pub no_indexing: bool,
}

const DEFAULT_BASIC_SPA: BasicSPA = BasicSPA {
    slug: "",
    page_title: "",
    page_description: "",
    only_follow: false,
    no_indexing: false,
};

const MDN_PLUS_TITLE: &str = "MDN Plus";
const OBSERVATORY_TITLE_FULL: &str = "HTTP Observatory | MDN";

const OBSERVATORY_DESCRIPTION: &str =
"Test your site’s HTTP headers, including CSP and HSTS, to find security problems and get actionable recommendations to make your website more secure. Test other websites to see how you compare.";

static BASIC_SPAS: Map<&'static str, BasicSPA> = phf_map!(
    "play" => BasicSPA {
        slug: "play",
        page_title: "Playground | MDN",
        ..DEFAULT_BASIC_SPA
    },
    "observatory" => BasicSPA {
        slug: "observatory",
        page_title: concat!("HTTP Header Security Test - ", OBSERVATORY_TITLE_FULL),
        page_description: OBSERVATORY_DESCRIPTION,
        ..DEFAULT_BASIC_SPA
    },
    "observatory/analyze" => BasicSPA {
        slug: "observatory/analyze",
        page_title: concat!("Scan results - ", OBSERVATORY_TITLE_FULL),
        page_description: OBSERVATORY_DESCRIPTION,
        no_indexing: true,
        ..DEFAULT_BASIC_SPA
    },
    "observatory/docs/tests_and_scoring" => BasicSPA {
        slug: "observatory/docs/tests_and_scoring",
        page_title: concat!("Tests & Scoring - ", OBSERVATORY_TITLE_FULL),
        page_description: OBSERVATORY_DESCRIPTION,
        ..DEFAULT_BASIC_SPA
    },
    "observatory/docs/faq" => BasicSPA {
        slug: "observatory/docs/faq",
        page_title: concat!("FAQ - ", OBSERVATORY_TITLE_FULL),
        page_description: OBSERVATORY_DESCRIPTION,
        ..DEFAULT_BASIC_SPA
    },
    "search" => BasicSPA {
        slug: "search",
        page_title: "Search",
        only_follow: true,
        ..DEFAULT_BASIC_SPA
    },
    "plus" => BasicSPA {
        slug: "plus",
        page_title: MDN_PLUS_TITLE,
        ..DEFAULT_BASIC_SPA
    },
    "plus/ai-help" => BasicSPA {
        slug: "plus/ai-help",
        page_title: concat!("AI Help | ", MDN_PLUS_TITLE),
        ..DEFAULT_BASIC_SPA
    },
    "plus/collections" => BasicSPA {
        slug: "plus/collections",
        page_title: concat!("Collections | ", MDN_PLUS_TITLE),
        no_indexing: true,
        ..DEFAULT_BASIC_SPA
    },
    "plus/collections/frequently_viewed" => BasicSPA {
        slug: "plus/collections/frequently_viewed",
        page_title: concat!("Frequently viewed articles | ", MDN_PLUS_TITLE),
        no_indexing: true,
        ..DEFAULT_BASIC_SPA
    },
    "plus/updates" => BasicSPA {
        slug: "plus/updates",
        page_title: concat!("Updates | ", MDN_PLUS_TITLE),
        ..DEFAULT_BASIC_SPA
    },
    "plus/settings" => BasicSPA {
        slug: "plus/settings",
        page_title: concat!("Settings | ", MDN_PLUS_TITLE),
        no_indexing: true,
        ..DEFAULT_BASIC_SPA
    },
    "about" => BasicSPA {
        slug: "about",
        page_title: "About MDN",
        ..DEFAULT_BASIC_SPA
    },
    "community" => BasicSPA {
        slug: "community",
        page_title: "Contribute to MDN",
        ..DEFAULT_BASIC_SPA
    },
    "advertising" => BasicSPA {
        slug: "advertising",
        page_title: "Advertise with us",
        ..DEFAULT_BASIC_SPA
    },
    "newsletter" => BasicSPA {
        slug: "newsletter",
        page_title: "Stay Informed with MDN",
        ..DEFAULT_BASIC_SPA
    },
);
