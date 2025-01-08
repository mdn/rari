use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{NaiveDate, NaiveDateTime};
use rari_md::m2h;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::globals::blog_root;
use rari_types::locale::Locale;
use rari_types::RariEnv;
use rari_utils::io::read_to_string;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cached_readers::{blog_author_by_name, blog_files};
use crate::error::DocError;
use crate::pages::json::{PrevNextBySlug, SlugNTitle};
use crate::pages::page::{Page, PageCategory, PageLike, PageReader};
use crate::resolve::build_url;
use crate::utils::{calculate_read_time_minutes, locale_and_typ_from_path, modified_dt, split_fm};

#[derive(Clone, Debug, Default)]
pub struct Author {
    pub frontmatter: AuthorFrontmatter,
    pub path: PathBuf,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct AuthorLink {
    pub name: Option<String>,
    pub link: Option<String>,
    pub avatar_url: Option<String>,
}

impl AuthorLink {
    pub fn from_author(author: &Author, name: &str) -> Self {
        AuthorLink {
            name: author.frontmatter.name.clone(),
            link: author.frontmatter.link.clone(),
            avatar_url: author.frontmatter.avatar.as_ref().map(|avatar| {
                format!(
                    "/{}/blog/author/{name}/{avatar}",
                    Locale::default().as_url_str()
                )
            }),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct AuthorFrontmatter {
    pub name: Option<String>,
    pub link: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(default)]
pub struct AuthorMetadata {
    pub name: String,
    pub link: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(default)]
pub struct BlogImage {
    pub file: String,
    pub alt: Option<String>,
    pub source: Option<AuthorMetadata>,
    pub creator: Option<AuthorMetadata>,
}

#[derive(Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlogMeta {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub image: BlogImage,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub keywords: String,
    pub sponsored: bool,
    #[serde(serialize_with = "modified_dt")]
    pub date: NaiveDateTime,
    pub author: AuthorLink,
    #[serde(skip_serializing_if = "PrevNextBySlug::is_none")]
    pub links: PrevNextBySlug,
    pub read_time: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct BlogPostBuildMeta {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub image: BlogImage,
    pub keywords: String,
    pub sponsored: bool,
    pub published: bool,
    pub date: NaiveDate,
    pub author: String,
    pub url: String,
    pub full_path: PathBuf,
    pub path: PathBuf,
    pub read_time: usize,
}

impl From<&BlogPostBuildMeta> for BlogMeta {
    fn from(value: &BlogPostBuildMeta) -> Self {
        let BlogPostBuildMeta {
            slug,
            title,
            description,
            image,
            keywords,
            sponsored,
            date,
            author,
            url,
            read_time,
            ..
        } = value.to_owned();
        let links = prev_next(&url);
        Self {
            slug,
            title,
            description,
            image,
            keywords,
            sponsored,
            date: NaiveDateTime::from(date),
            read_time,
            author: blog_author_by_name(&author)
                .map(|a| AuthorLink::from_author(&a, &author))
                .unwrap_or(AuthorLink {
                    name: Some(author),
                    ..Default::default()
                }),
            links,
        }
    }
}

impl BlogPostBuildMeta {
    pub fn from_fm(
        fm: BlogPostFrontmatter,
        full_path: impl Into<PathBuf>,
        read_time: usize,
    ) -> Result<Self, DocError> {
        let full_path = full_path.into();
        let BlogPostFrontmatter {
            slug,
            title,
            description,
            image,
            keywords,
            sponsored,
            published,
            date,
            author,
        } = fm;
        let (locale, _) = locale_and_typ_from_path(&full_path)
            .unwrap_or((Default::default(), PageCategory::BlogPost));
        let url = build_url(&slug, locale, PageCategory::BlogPost)?;
        let path = full_path
            .strip_prefix(blog_root().ok_or(DocError::NoBlogRoot)?)?
            .to_path_buf();
        Ok(Self {
            url,
            slug,
            title,
            description,
            image,
            keywords,
            sponsored,
            published,
            date,
            author,
            full_path,
            path,
            read_time,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default)]
pub struct BlogPostFrontmatter {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub image: BlogImage,
    pub keywords: String,
    pub sponsored: bool,
    pub published: bool,
    pub date: NaiveDate,
    pub author: String,
}

#[derive(Debug, Clone)]
pub struct BlogPost {
    pub meta: BlogPostBuildMeta,
    raw: String,
    content_start: usize,
}

impl BlogPost {
    pub fn page_from_url(url: &str) -> Option<Page> {
        let _ = blog_root()?;
        blog_files().posts.get(&url.to_ascii_lowercase()).cloned()
    }
}

impl PageReader<Page> for BlogPost {
    fn read(path: impl Into<PathBuf>, _: Option<Locale>) -> Result<Page, DocError> {
        read_blog_post(path).map(Arc::new).map(Page::BlogPost)
    }
}

impl PageLike for BlogPost {
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
            locale: Default::default(),
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

    fn fm_offset(&self) -> usize {
        self.raw[..self.content_start].lines().count()
    }
}

fn read_blog_post(path: impl Into<PathBuf>) -> Result<BlogPost, DocError> {
    let full_path = path.into();
    let raw = read_to_string(&full_path)?;
    let (fm, content_start) = split_fm(&raw);
    let fm = fm.ok_or(DocError::NoFrontmatter)?;
    let fm: BlogPostFrontmatter = serde_yaml_ng::from_str(fm)?;

    let read_time = calculate_read_time_minutes(&raw[content_start..]);
    Ok(BlogPost {
        meta: BlogPostBuildMeta::from_fm(fm, full_path, read_time)?,
        raw,
        content_start,
    })
}

fn prev_next(url: &str) -> PrevNextBySlug {
    let sorted_meta = &blog_files().sorted_meta;
    if let Some(i) = sorted_meta.iter().position(|m| m.url == url) {
        PrevNextBySlug {
            previous: if i > 0 {
                sorted_meta.get(i - 1).map(|m| SlugNTitle {
                    slug: m.slug.clone(),
                    title: m.title.clone(),
                })
            } else {
                None
            },
            next: if i < sorted_meta.len() - 1 {
                sorted_meta.get(i + 1).map(|m| SlugNTitle {
                    slug: m.slug.clone(),
                    title: m.title.clone(),
                })
            } else {
                None
            },
        }
    } else {
        Default::default()
    }
}
